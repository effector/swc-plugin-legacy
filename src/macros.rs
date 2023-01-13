#[macro_export]
macro_rules! obj_lit {
    ($($json:tt)+) => {
        $crate::obj_lit_internal!($($json)+)
    };
}

#[macro_export]
macro_rules! obj_lit_internal {
    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        $object.props.push(
            ::swc_core::ecma::ast::PropOrSpread::Prop(
                Box::new(::swc_core::ecma::ast::Prop::KeyValue(
                    ::swc_core::ecma::ast::KeyValueProp {
                        key: ::swc_core::ecma::ast::PropName::Ident(
                            ::swc_core::ecma::utils::quote_ident!(($($key)+))
                        ),
                        value: Box::new(
                            ::swc_core::ecma::ast::Expr::Lit(
                                ::swc_core::ecma::ast::Lit::Str(
                                    $value
                                )
                            )
                        )
                    }
                ))
            )
        );

        $crate::obj_lit_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        $object.props.push(
            ::swc_core::ecma::ast::PropOrSpread::Prop(
                Box::new(::swc_core::ecma::ast::Prop::KeyValue(
                    ::swc_core::ecma::ast::KeyValueProp {
                        key: ::swc_core::ecma::ast::PropName::Ident(
                            ::swc_core::ecma::utils::quote_ident!(($($key)+))
                        ),
                        value: Box::new(
                            ::swc_core::ecma::ast::Expr::Lit(
                                ::swc_core::ecma::ast::Lit::Str(
                                    $value
                                )
                            )
                        )
                    }
                ))
            )
        );
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::obj_lit_internal!(
            @object $object [$($key)+] ($crate::obj_lit_internal!($value)) , $($rest)*
        );
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        $crate::obj_lit_internal!(@object $object [$($key)+] ($crate::obj_lit_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::obj_lit_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::obj_lit_internal!();
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        $crate::obj_lit_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::obj_lit_internal!(
            @object $object ($($key)* $tt) ($($rest)*) ($($rest)*)
        );
    };

    ({ $($tt:tt)+ }) => {
        {
            let mut object = (::swc_core::ecma::ast::ObjectLit {
                span: ::swc_core::common::DUMMY_SP,
                props: vec![]
            });

            $crate::obj_lit_internal!(@object object () ($($tt)+) ($($tt)+));

            object
        }
    };

    ($other:expr) => {
        ::swc_core::ecma::ast::Str::from($other)
    };
}
