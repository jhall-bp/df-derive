use df_derive::ToDataFrame;

#[derive(ToDataFrame)]
struct Inner {
    value: i32,
}

#[derive(ToDataFrame)]
struct EmptyFlattenPrefix {
    #[df_derive(flatten(prefix = ""))]
    empty: Inner,
}

#[derive(ToDataFrame)]
struct DuplicateFlattenPrefix {
    #[df_derive(flatten(prefix = "a", prefix = "b"))]
    duplicate: Inner,
}

#[derive(ToDataFrame)]
struct UnknownFlattenPrefixKey {
    #[df_derive(flatten(name = "a"))]
    unknown: Inner,
}

fn main() {}
