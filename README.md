# tokio-async-drop

This is a quick and dirty "hack" (not really an hack, since it's the [desired behaviour](https://github.com/tokio-rs/tokio/issues/5843) of used components) to allow async drop in a tokio multithreaded runtime.

The same approach could potentially be used to allow async code in many other situations, e.g. inside a [`once_cell::sync::Lazy`](https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html) static variable.

## Example

```rust
struct Foo<'a> {
    inner: &'a mut u8,
}

impl<'a> Foo<'a> {
    async fn bar(&mut self) {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        *self.inner = 1;
    }
}

impl<'a> Drop for Foo<'a> {
    fn drop(&mut self) {
        tokio_async_drop!({
            self.bar().await;
        });
    }
}

#[tokio::main]
async fn main() {
    let mut semaphore = 0;
    let f = Foo {
        inner: &mut semaphore,
    };
    drop(f);
    assert_eq!(semaphore, 1);
}
```
