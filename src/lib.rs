#[macro_export]
macro_rules! tokio_async_drop {
    ($drop:block) => {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { $drop });
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic(expected = "can call blocking only when running on the multi-threaded runtime")]
    fn current_thread() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                test_core();
            });
    }

    #[test]
    fn multi_thread() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                test_core();
            });
    }
}
