use leptoaster::*;
use leptos::task::spawn_local;

#[derive(Clone)]
pub struct MyToaster
{
    inner: ToasterContext,
}

impl MyToaster
{
    pub fn new() -> Self
    {
        Self { inner: expect_toaster() }
    }

    pub fn success(&self, msg: &str)
    {
        let msg = msg.to_owned();
        let toaster = self.inner.clone();

        spawn_local(async move {
            toaster.toast(
                ToastBuilder::new(msg)
                .with_level(ToastLevel::Success)
                .with_position(ToastPosition::BottomRight),
            );
        });
    }

    pub fn error(&self, msg: &str)
    {
        let msg = msg.to_owned();
        let toaster = self.inner.clone();

        spawn_local(async move {
            toaster.toast(
                ToastBuilder::new(msg)
                .with_level(ToastLevel::Error)
                .with_position(ToastPosition::BottomRight),
            );
        });
    }

    pub fn info(&self, msg: &str)
    {
        let msg = msg.to_owned();
        let toaster = self.inner.clone();

        spawn_local(async move {
            toaster.toast(
                ToastBuilder::new(msg)
                .with_level(ToastLevel::Info)
                .with_position(ToastPosition::BottomRight),
            );
        });
    }
}
