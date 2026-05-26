use df_derive::ToDataFrame;

#[derive(ToDataFrame)]
struct Inner {
    value: i32,
}

#[derive(ToDataFrame)]
struct PrimitiveFlatten {
    #[df_derive(flatten)]
    primitive: i32,
}

#[derive(ToDataFrame)]
struct OptionalFlatten {
    #[df_derive(flatten)]
    optional: Option<Inner>,
}

#[derive(ToDataFrame)]
struct BoxedOptionalFlatten {
    #[df_derive(flatten)]
    boxed_optional: Box<Option<Inner>>,
}

#[derive(ToDataFrame)]
struct OptionalBoxedFlatten {
    #[df_derive(flatten)]
    optional_boxed: Option<Box<Inner>>,
}

#[derive(ToDataFrame)]
struct VectorFlatten {
    #[df_derive(flatten)]
    vector: Vec<Inner>,
}

#[derive(ToDataFrame)]
struct TupleFlatten {
    #[df_derive(flatten)]
    tuple: (i32, String),
}

fn main() {}
