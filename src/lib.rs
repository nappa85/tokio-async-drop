#[macro_export]
macro_rules! tokio_async_drop {
    ($drop:block) => {
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            debug_assert!(handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread);
            handle.block_on(async { $drop });
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Foo<'a> {
        inner: &'a mut u8,
    }

    impl<'a> Foo<'a> {
        async fn bar(&mut self) {
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

    fn test_core() {
        let mut semaphore = 0;
        let f = Foo {
            inner: &mut semaphore,
        };
        drop(f);
        assert_eq!(semaphore, 1);
    }

    #[test]
    #[should_panic(
        expected = "there is no reactor running, must be called from the context of a Tokio 1.x runtime"
    )]
    fn no_runtime() {
        test_core();
    }

    #[test]
    fn it_works() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                test_core();
            });
    }
}
