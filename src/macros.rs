macro_rules! variant_to_string {
    ($enum:ident::$variant:ident) => {
        stringify!($variant).to_lowercase()
    };
}
