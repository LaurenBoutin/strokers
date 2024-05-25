use async_trait::async_trait;
use strokers_core::{AxisDescriptor, Movement, Stroker};
pub use strokers_device_debug as debug;
pub use strokers_device_tcode as tcode;

/// Wrapper for a [`Box`]ed [`Stroker`].
/// This makes it easier to support any type of stroker in your application.
///
/// Get one with [`crate::open_stroker`].
pub struct AnyStroker {
    inner: Box<dyn Stroker + Send + 'static>,
}

impl AnyStroker {
    pub fn new(stroker: impl Stroker + Send + 'static) -> AnyStroker {
        AnyStroker {
            inner: Box::new(stroker),
        }
    }
}

#[async_trait]
impl Stroker for AnyStroker {
    fn axes(&mut self) -> Vec<AxisDescriptor> {
        self.inner.axes()
    }

    async fn stop(&mut self) -> eyre::Result<()> {
        self.inner.stop().await
    }

    async fn movement(&mut self, movement: Movement) -> eyre::Result<()> {
        self.inner.movement(movement).await
    }

    fn description(&mut self) -> eyre::Result<Option<String>> {
        self.inner.description()
    }
}
