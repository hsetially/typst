//! Helper types and macros for creating custom functions.


/// Defines function types concisely.
#[macro_export]
macro_rules! function {
    // Parse a unit struct.
    ($(#[$outer:meta])* pub struct $type:ident; $($rest:tt)*) => {
        $(#[$outer])* pub struct $type;
        function!(@meta $type | $($rest)*);
    };

    // Parse a tuple struct.
    ($(#[$outer:meta])* pub struct $type:ident($($fields:tt)*); $($rest:tt)*) => {
        $(#[$outer])* pub struct $type($($fields)*);
        function!(@meta $type | $($rest)*);
    };

    // Parse a struct with fields.
    ($(#[$outer:meta])* pub struct $type:ident { $($fields:tt)* } $($rest:tt)*) => {
        $(#[$outer])* pub struct $type { $($fields)* }
        function!(@meta $type | $($rest)*);
    };

    // Parse an enum.
    ($(#[$outer:meta])* pub enum $type:ident { $($fields:tt)* } $($rest:tt)*) => {
        $(#[$outer])* pub enum $type { $($fields)* }
        function!(@meta $type | $($rest)*);
    };

    // Parse a metadata type definition.
    (@meta $type:ident | type Meta = $meta:ty; $($rest:tt)*) => {
        function!(@parse $type $meta | $($rest)*);
    };

    // Set the metadata to `()` if there is no type definition.
    (@meta $type:ident | $($rest:tt)*) => {
        function!(@parse $type () | $($rest)*);
    };

    // Parse a `parse(default)`.
    (@parse $type:ident $meta:ty | parse(default) $($rest:tt)*) => {
        function!(@parse $type $meta |
            parse(_args, _body, _ctx, _meta) { Default::default() }
            $($rest)*
        );
    };

    // (0-arg) Parse a parse-definition without arguments.
    (@parse $type:ident $meta:ty | parse() $code:block $($rest:tt)*) => {
        function!(@parse $type $meta | parse(_args, _body, _ctx, _meta) $code $($rest)*);
    };

    // (1-arg) Parse a parse-definition with only the first argument.
    (@parse $type:ident $meta:ty | parse($header:ident) $code:block $($rest:tt)*) => {
        function!(@parse $type $meta | parse($header, _body, _ctx, _meta) $code $($rest)*);
    };

    // (2-arg) Parse a parse-definition with only the first two arguments.
    (@parse $type:ident $meta:ty |
        parse($header:ident, $body:pat) $code:block $($rest:tt)*
    ) => {
        function!(@parse $type $meta | parse($header, $body, _ctx, _meta) $code $($rest)*);
    };

    // (3-arg) Parse a parse-definition with only the first three arguments.
    (@parse $type:ident $meta:ty |
        parse($header:ident, $body:pat, $ctx:pat) $code:block $($rest:tt)*
    ) => {
        function!(@parse $type $meta | parse($header, $body, $ctx, _meta) $code $($rest)*);
    };

    // (4-arg) Parse a parse-definition with all four arguments.
    (@parse $type:ident $meta:ty |
        parse($header:ident, $body:pat, $ctx:pat, $metadata:pat) $code:block
        $($rest:tt)*
    ) => {
        impl $crate::func::ParseFunc for $type {
            type Meta = $meta;

            fn parse(
                header: $crate::syntax::FuncHeader,
                $body: Option<&str>,
                $ctx: $crate::syntax::ParseContext,
                $metadata: Self::Meta,
            ) -> $crate::syntax::ParseResult<Self> where Self: Sized {
                #[allow(unused_mut)]
                let mut $header = header;
                let val = $code;
                if !$header.args.is_empty() {
                    return Err($crate::TypesetError::with_message("unexpected arguments"));
                }
                Ok(val)
            }
        }

        function!(@layout $type | $($rest)*);
    };

    // (0-arg) Parse a layout-definition without arguments.
    (@layout $type:ident | layout() $code:block) => {
        function!(@layout $type | layout(self, _ctx) $code);
    };

    // (1-arg) Parse a layout-definition with only the first argument.
    (@layout $type:ident | layout($this:ident) $code:block) => {
        function!(@layout $type | layout($this, _ctx) $code);
    };

    // (2-arg) Parse a layout-definition with all arguments.
    (@layout $type:ident | layout($this:ident, $ctx:pat) $code:block) => {
        impl $crate::func::LayoutFunc for $type {
            fn layout<'a, 'life0, 'life1, 'async_trait>(
                &'a $this,
                $ctx: $crate::layout::LayoutContext<'life0, 'life1>
            ) -> std::pin::Pin<Box<dyn std::future::Future<
                Output = $crate::layout::LayoutResult<
                    $crate::func::Commands<'a>>
                > + 'async_trait
            >>
            where
                'a: 'async_trait,
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait,
            {
                #[allow(unreachable_code)]
                Box::pin(async move { Ok($code) })
            }
        }
    };
}

/// Parse the body of a function.
///
/// - If the function does not expect a body, use `parse!(forbidden: body)`.
/// - If the function can have a body, use `parse!(optional: body, ctx)`.
/// - If the function must have a body, use `parse!(expected: body, ctx)`.
#[macro_export]
macro_rules! parse {
    (forbidden: $body:expr) => {
        if $body.is_some() {
            return Err($crate::TypesetError::with_message("unexpected body"));
        }
    };

    (optional: $body:expr, $ctx:expr) => (
        if let Some(body) = $body {
            Some($crate::syntax::parse(body, $ctx).0)
        } else {
            None
        }
    );

    (expected: $body:expr, $ctx:expr) => (
        if let Some(body) = $body {
            $crate::syntax::parse(body, $ctx).0
        } else {
            Err($crate::TypesetError::with_message("unexpected body"))
        }
    )
}

/// Early-return with a formatted typesetting error or construct an error
/// expression.
#[macro_export]
macro_rules! error {
    (@unexpected_argument) => (error!(@"unexpected argument"));
    (@$($tts:tt)*) => ($crate::TypesetError::with_message(format!($($tts)*)));
    ($($tts:tt)*) => (return Err(error!(@$($tts)*)););
}
