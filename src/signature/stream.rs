use bytes::{BufMut, Bytes, BytesMut};
use futures::{future, stream, Stream, StreamExt};
use pin_project::pin_project;
use tokio::io::{AsyncRead, ReadBuf};

use std::fmt;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream of bytes.
#[pin_project]
pub struct ByteStream {
    size_hint: Option<usize>,
    #[pin]
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Sync + 'static>>,
}

impl ByteStream {
    /// Create a new `ByteStream` by wrapping a `futures` stream.
    pub fn new<S>(stream: S) -> ByteStream
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send + Sync + 'static,
    {
        ByteStream {
            size_hint: None,
            inner: Box::pin(stream),
        }
    }

    /// Creates a new `ByteStream` by wrapping a `futures` stream. Allows for the addition of a
    /// size_hint to satisy S3's `PutObject` API.
    pub fn new_with_size<S>(stream: S, size_hint: usize) -> ByteStream
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send + Sync + 'static,
    {
        ByteStream {
            size_hint: Some(size_hint),
            inner: Box::pin(stream),
        }
    }

    pub(crate) fn size_hint(&self) -> Option<usize> {
        self.size_hint
    }

    /// Return an implementation of `AsyncRead` that uses async i/o to consume the stream.
    pub fn into_async_read(self) -> impl AsyncRead + Send + Sync {
        ImplAsyncRead::new(self.inner)
    }

    /// Return an implementation of `Read` that uses blocking i/o to consume the stream.
    pub fn into_blocking_read(self) -> impl io::Read + Send + Sync {
        ImplBlockingRead::new(self.inner)
    }
}

impl From<Vec<u8>> for ByteStream {
    fn from(buf: Vec<u8>) -> ByteStream {
        ByteStream {
            size_hint: Some(buf.len()),
            inner: Box::pin(stream::once(async move { Ok(Bytes::from(buf)) })),
        }
    }
}

impl fmt::Debug for ByteStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<ByteStream size_hint={:?}>", self.size_hint)
    }
}

impl Stream for ByteStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

#[pin_project]
struct ImplAsyncRead {
    buffer: BytesMut,
    #[pin]
    stream:
        futures::stream::Fuse<Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Sync>>>,
}

impl ImplAsyncRead {
    fn new(stream: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Sync>>) -> Self {
        ImplAsyncRead {
            buffer: BytesMut::new(),
            stream: stream.fuse(),
        }
    }
}

impl AsyncRead for ImplAsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        if this.buffer.is_empty() {
            match futures::ready!(this.stream.poll_next(cx)) {
                None => return Poll::Ready(Ok(())),
                Some(Err(e)) => return Poll::Ready(Err(e)),
                Some(Ok(bytes)) => {
                    this.buffer.put(bytes);
                }
            }
        }
        let available = std::cmp::min(buf.remaining(), this.buffer.len());
        let bytes = this.buffer.split_to(available);
        let mut left = buf.take(available);
        left.put_slice(&bytes[..available]);
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
struct ImplBlockingRead {
    #[pin]
    inner: ImplAsyncRead,
}

impl ImplBlockingRead {
    fn new(stream: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Sync>>) -> Self {
        ImplBlockingRead {
            inner: ImplAsyncRead::new(stream),
        }
    }
}

impl io::Read for ImplBlockingRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rt = tokio::runtime::Runtime::new()?;
        let mut rb = ReadBuf::new(buf);
        rt.block_on(future::poll_fn(|cx| match tokio::io::AsyncRead::poll_read(
            Pin::new(&mut self.inner),
            cx,
            &mut rb,
        ) {
            std::task::Poll::Ready(_) => Poll::Ready(Ok(rb.filled().len())),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }))
    }
}
