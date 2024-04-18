use axum_server::accept::Accept;
use futures_util::{ready, Future};
use http_body::{Body, Frame};
use hyper::Response;
use pin_project_lite::pin_project;
use std::time::Duration;
use std::{
    future::Ready,
    io::ErrorKind,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::{Instant, Sleep},
};
use tower::Service;

const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct TimeoutAcceptor;

impl<I, S> Accept<I, S> for TimeoutAcceptor {
    type Stream = TimeoutStream<I>;
    type Service = TimeoutService<S>;
    type Future = Ready<std::io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let (tx, rx) = mpsc::unbounded_channel();

        let stream = TimeoutStream::new(stream, TIMEOUT, rx);
        let service = TimeoutService::new(service, tx);

        std::future::ready(Ok((stream, service)))
    }
}

#[derive(Clone)]
pub struct TimeoutService<S> {
    inner: S,
    sender: UnboundedSender<TimerSignal>,
}

impl<S> TimeoutService<S> {
    fn new(inner: S, sender: UnboundedSender<TimerSignal>) -> Self {
        Self { inner, sender }
    }
}

impl<S, B, Request> Service<Request> for TimeoutService<S>
where
    S: Service<Request, Response = Response<B>>,
{
    type Response = Response<TimeoutBody<B>>;
    type Error = S::Error;
    type Future = TimeoutServiceFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // send timer wait signal
        let _ = self.sender.send(TimerSignal::Wait);

        TimeoutServiceFuture::new(self.inner.call(req), self.sender.clone())
    }
}

pin_project! {
    pub struct TimeoutServiceFuture<F> {
        #[pin]
        inner: F,
        sender: Option<UnboundedSender<TimerSignal>>,
    }
}

impl<F> TimeoutServiceFuture<F> {
    fn new(inner: F, sender: UnboundedSender<TimerSignal>) -> Self {
        Self {
            inner,
            sender: Some(sender),
        }
    }
}

impl<F, B, E> Future for TimeoutServiceFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<TimeoutBody<B>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx).map(|result| {
            result.map(|response| {
                response.map(|body| {
                    TimeoutBody::new(body, this.sender.take().expect("future polled after ready"))
                })
            })
        })
    }
}

enum TimerSignal {
    Wait,
    Reset,
}

pin_project! {
    pub struct TimeoutBody<B> {
        #[pin]
        inner: B,
        sender: UnboundedSender<TimerSignal>,
    }
}

impl<B> TimeoutBody<B> {
    fn new(inner: B, sender: UnboundedSender<TimerSignal>) -> Self {
        Self { inner, sender }
    }
}

impl<B: Body> Body for TimeoutBody<B> {
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let option = ready!(this.inner.poll_frame(cx));

        if option.is_none() {
            let _ = this.sender.send(TimerSignal::Reset);
        }

        Poll::Ready(option)
    }

    fn is_end_stream(&self) -> bool {
        let is_end_stream = self.inner.is_end_stream();

        if is_end_stream {
            let _ = self.sender.send(TimerSignal::Reset);
        }

        is_end_stream
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

pub struct TimeoutStream<IO> {
    inner: IO,
    // hyper requires unpin
    sleep: Pin<Box<Sleep>>,
    duration: Duration,
    waiting: bool,
    receiver: UnboundedReceiver<TimerSignal>,
    finished: bool,
}

impl<IO> TimeoutStream<IO> {
    fn new(inner: IO, duration: Duration, receiver: UnboundedReceiver<TimerSignal>) -> Self {
        Self {
            inner,
            sleep: Box::pin(tokio::time::sleep(duration)),
            duration,
            waiting: false,
            receiver,
            finished: false,
        }
    }
}

impl<IO: AsyncRead + Unpin> AsyncRead for TimeoutStream<IO> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if !self.finished {
            match Pin::new(&mut self.receiver).poll_recv(cx) {
                // reset the timer
                Poll::Ready(Some(TimerSignal::Reset)) => {
                    self.waiting = false;

                    let deadline = Instant::now() + self.duration;
                    self.sleep.as_mut().reset(deadline);
                }
                // enter waiting mode (for response body last chunk)
                Poll::Ready(Some(TimerSignal::Wait)) => self.waiting = true,
                Poll::Ready(None) => self.finished = true,
                Poll::Pending => (),
            }
        }

        if !self.waiting {
            // return error if timer is elapsed
            if let Poll::Ready(()) = self.sleep.as_mut().poll(cx) {
                return Poll::Ready(Err(std::io::Error::new(
                    ErrorKind::TimedOut,
                    "request header read timed out",
                )));
            }
        }

        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<IO: AsyncWrite + Unpin> AsyncWrite for TimeoutStream<IO> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
}
