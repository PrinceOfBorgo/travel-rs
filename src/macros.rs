macro_rules! variant_to_string {
    ($enum:ident::$variant:ident) => {
        stringify!($variant).to_lowercase()
    };
}

#[cfg(test)]
// This macro is used to define asynchronous test functions with a specific configuration.
// It simplifies the creation of test functions by automatically applying the `tokio::test`
// attribute with the specified flavor and worker threads.
macro_rules! test {
    ($title:ident, $($stmt:stmt)*) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn $title() {
            $($stmt)*
        }
    };
}
