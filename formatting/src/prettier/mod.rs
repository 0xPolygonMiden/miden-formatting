//! This module provides a generic pretty printer implementation of the "Prettier" variety, i.e.
//! based on the design described by Philip Wadler in
//! [_A prettier printer_](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf).
//!
//! This can be used to implement a pretty printer for arbitrarily complex and nested data
//! structures and syntaxes, and handles the tricky details of indentation, when to break lines,
//! etc.
//!
//! See the [PrettyPrint] trait for more on how to use this module.
mod document;
mod print;
#[cfg(test)]
mod tests;

use alloc::string::String;
use core::fmt;

pub use self::document::{concat, const_text, display, flatten, indent, nl, split, text, Document};

/// The [PrettyPrint] trait is used as a building block for pretty printing data or syntax trees,
/// as commonly seen in tools like Prettier.
///
/// It relies on providing the desired layout for a given type in the form of a [Document]. The
/// pretty printer algorithm then renders this document to an output stream, using details of
/// the output and the document to drive the choices it makes as it displays the document.
///
/// To get started, simply implement [PrettyPrint::render] for the type you wish to have pretty
/// printed. You can then call [PrettyPrint::to_pretty_string] to obtain a [String] containing
/// the pretty printed output, or if you have a [core::fmt::Formatter] handy, you can pretty
/// print directly to that formatter using [PrettyPrint::pretty_print].
///
/// # Example
///
/// The following is the AST for a simple expression language with a couple ops, that we wish to
/// implement a pretty printer for. Let's take a look at how that is done, and how we can make
/// use of the various document constructors to acheive various effects:
///
/// ```rust
/// use miden_formatting::prettier::{self, PrettyPrint, Document};
///
/// pub enum Expr {
///     Term(Term),
///     Binary(BinaryOp),
/// }
///
/// pub struct BinaryOp {
///     pub op: Op,
///     pub lhs: Box<Expr>,
///     pub rhs: Box<Expr>,
/// }
///
/// #[derive(Copy, Clone)]
/// pub enum Op {
///     Add,
///     Sub,
///     Mul,
/// }
///
/// pub enum Term {
///     Var(String),
///     Num(isize),
/// }
///
/// impl PrettyPrint for Expr {
///     fn render(&self) -> Document {
///         match self {
///             Self::Term(term) => term.render(),
///             Self::Binary(expr) => expr.render(),
///         }
///     }
/// }
///
/// /// We can trivially implement Display for our AST with our PrettyPrint impl
/// impl core::fmt::Display for Expr {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         self.pretty_print(f)
///     }
/// }
///
/// impl PrettyPrint for BinaryOp {
///     fn render(&self) -> Document {
///         // Bring all of the document constructors into scope
///         use prettier::*;
///
///         let maybe_wrap = move |expr: &Expr| -> Document {
///             match expr {
///                 Expr::Term(term) => term.render(),
///                 Expr::Binary(expr) => '(' + expr.render() + ')',
///             }
///         };
///
///         // When the printer runs out of space to hold more of the document on a single line,
///         // it will prefer to introduce breaks at `nl`, but may also choose to break at any
///         // point where two documents are joined together. We can guide this behavior by providing
///         // a choice between two documents: the first being the single-line layout, and the second
///         // being the multi-line layout. The printer will choose the single-line layout unless it
///         // has insufficient space for it, in which case it will choose the multi-line layout instead.
///         let single_line = self.lhs.render()
///                 + ' '
///                 + display(self.op)
///                 + ' '
///                 + maybe_wrap(&self.rhs);
///         // Here, we're choosing to break after the operator, indent 4 spaces, then continue to
///         // print the remainder of the expression, e.g:
///         //
///         //     $a + ($b * ($c -
///         //         256))
///         let multi_line =
///                 indent(4, flatten(self.lhs.render() + ' ' + display(self.op))
///                 + nl()
///                 + maybe_wrap(&self.rhs)
///                 );
///         single_line | multi_line
///     }
/// }
///
/// impl PrettyPrint for Term {
///     fn render(&self) -> Document {
///         use prettier::*;
///         // NOTE: We could have just used a Display impl for Term, but in more complex syntaxes
///         // terms might have aggregate data structures and things of that nature, where more
///         // complex pretty printing is desired. For now, this just demonstrates how you can
///         // implement PrettyPrint for types you don't control with custom formatting.
///         match self {
///             Self::Var(v) => text(format!("${v}")),
///             Self::Num(n) => display(*n),
///         }
///     }
/// }
///
/// /// Rather than implement both PrettyPrint and Display for things which reduce to keywords,
/// /// integers, etc., you can simply delegate to the Display implementation when building the
/// /// higher-level PrettyPrint impls using the `display` helper.
/// impl core::fmt::Display for Op {
///     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
///         match self {
///             Self::Add => f.write_str("+"),
///             Self::Sub => f.write_str("-"),
///             Self::Mul => f.write_str("*"),
///         }
///     }
/// }
/// ```
///
/// See the documentation for the various [Document] constructors for more information on their
/// usage and semantics. I would recommend starting with [indent], as that is a common one that
/// you will want to use.
pub trait PrettyPrint {
    /// The core of the [PrettyPrint] functionality.
    ///
    /// When called, the implementation must render a [Document] which represents the layout
    /// of the thing being pretty-printed. The rendered [Document] is then displayed via the
    /// pretty printer, using details about the output stream, and the structure of the document,
    /// to make decisions about when and where to introduce line breaks, etc.
    ///
    /// Implementations do not need to worry about or manage things like the width of the output,
    /// indentation level, etc. Instead the focus is purely on the layout, leaving the heavy
    /// lifting to the pretty printer.
    ///
    /// This method is the only one required to be implemented.
    fn render(&self) -> Document;

    /// Produce a [String] containing the results of pretty-printing this object.
    ///
    /// The string is formatted with an assumed width of 80 columns. If you wish to customize this,
    /// you should instead prefer to use [PrettyPrint::pretty_print], or if you have implemented
    /// [core::fmt::Display] for this type by delegating to [PrettyPrint::pretty_print], you can
    /// use the Rust formatting syntax to do this, e.g. `format!("{:width$}", self, width = 100)`
    fn to_pretty_string(&self) -> String {
        format!("{:width$}", Prettier(self), width = 80)
    }

    /// Pretty-print this object to the given [core::fmt::Formatter].
    ///
    /// You may implement [core::fmt::Display] for your type in terms of this function like so:
    ///
    /// ```rust,ignore
    /// impl fmt::Display for Foo {
    ///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    ///         self.pretty_print(f)
    ///     }
    /// }
    /// ```
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let doc = self.render();
        let width = f.width().unwrap_or(80);
        print::pretty_print(&doc, width, f)
    }
}

impl fmt::Display for dyn PrettyPrint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self, f)
    }
}

/// Generate an implementation of [PrettyPrint] for a given type by delegating to [core::fmt::Display].
///
/// # Example
///
/// ```rust,ignore
/// pretty_via_to_display!(Foo);
/// ```
#[macro_export]
macro_rules! pretty_via_display {
    ($name:ty) => {
        impl $crate::prettier::PrettyPrint for $name {
            fn render(&self) -> $crate::prettier::Document {
                $crate::prettier::display(*self)
            }
        }
    };
}

/// Generate an implementation of [PrettyPrint] for a given type by delegating to [alloc::string::ToString].
///
/// # Example
///
/// ```rust,ignore
/// pretty_via_to_string!(Foo);
/// ```
#[macro_export]
macro_rules! pretty_via_to_string {
    ($name:ty) => {
        impl $crate::prettier::PrettyPrint for $name {
            fn render(&self) -> $crate::prettier::Document {
                $crate::prettier::text(&**self)
            }
        }
    };
}

pretty_via_display!(bool);
pretty_via_display!(u8);
pretty_via_display!(i8);
pretty_via_display!(u16);
pretty_via_display!(i16);
pretty_via_display!(u32);
pretty_via_display!(i32);
pretty_via_display!(u64);
pretty_via_display!(i64);
pretty_via_display!(u128);
pretty_via_display!(i128);
pretty_via_display!(usize);
pretty_via_display!(isize);
pretty_via_display!(core::num::NonZeroU8);
pretty_via_display!(core::num::NonZeroI8);
pretty_via_display!(core::num::NonZeroU16);
pretty_via_display!(core::num::NonZeroI16);
pretty_via_display!(core::num::NonZeroU32);
pretty_via_display!(core::num::NonZeroI32);
pretty_via_display!(core::num::NonZeroU64);
pretty_via_display!(core::num::NonZeroI64);
pretty_via_display!(core::num::NonZeroU128);
pretty_via_display!(core::num::NonZeroI128);
pretty_via_display!(core::num::NonZeroUsize);
pretty_via_display!(core::num::NonZeroIsize);

impl<'a, T: ?Sized + PrettyPrint> PrettyPrint for &'a T {
    #[inline]
    fn render(&self) -> Document {
        (**self).render()
    }
    #[inline]
    fn to_pretty_string(&self) -> String {
        (**self).to_pretty_string()
    }
    #[inline]
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).pretty_print(f)
    }
}

impl PrettyPrint for str {
    fn render(&self) -> Document {
        self.lines()
            .map(text)
            .reduce(|acc, doc| match acc {
                Document::Empty => doc + nl(),
                other => other + doc + nl(),
            })
            .unwrap_or(Document::Empty)
    }
}

impl PrettyPrint for String {
    fn render(&self) -> Document {
        PrettyPrint::render(self.as_str())
    }
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self.as_str(), f)
    }
}

impl<'a> PrettyPrint for alloc::borrow::Cow<'a, str> {
    fn render(&self) -> Document {
        PrettyPrint::render(self.as_ref())
    }
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self.as_ref(), f)
    }
}

impl<T: PrettyPrint> PrettyPrint for alloc::boxed::Box<T> {
    fn render(&self) -> Document {
        PrettyPrint::render(self.as_ref())
    }
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self.as_ref(), f)
    }
}

impl<T: PrettyPrint> PrettyPrint for alloc::rc::Rc<T> {
    fn render(&self) -> Document {
        PrettyPrint::render(self.as_ref())
    }
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self.as_ref(), f)
    }
}

impl<T: PrettyPrint> PrettyPrint for alloc::sync::Arc<T> {
    fn render(&self) -> Document {
        PrettyPrint::render(self.as_ref())
    }
    fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        PrettyPrint::pretty_print(self.as_ref(), f)
    }
}

impl<T: PrettyPrint> PrettyPrint for alloc::vec::Vec<T> {
    fn render(&self) -> Document {
        let single = self.iter().fold(Document::Empty, |acc, e| match acc {
            Document::Empty => e.render(),
            acc => acc + ',' + ' ' + e.render(),
        });
        let multi = self.iter().fold(Document::Empty, |acc, e| match acc {
            Document::Empty => e.render(),
            acc => acc + ',' + nl() + e.render(),
        });
        let single_line = '[' + single + ']';
        let multi_line = '[' + indent(4, nl() + multi) + nl() + ']';
        single_line | multi_line
    }
}

impl<T: PrettyPrint> PrettyPrint for alloc::collections::BTreeSet<T> {
    fn render(&self) -> Document {
        let single = self.iter().fold(Document::Empty, |acc, e| match acc {
            Document::Empty => e.render(),
            acc => acc + ',' + ' ' + e.render(),
        });
        let multi = self.iter().fold(Document::Empty, |acc, e| match acc {
            Document::Empty => e.render(),
            acc => acc + ',' + nl() + e.render(),
        });
        let single_line = '{' + single + '}';
        let multi_line = '{' + indent(4, nl() + multi) + nl() + '}';
        single_line | multi_line
    }
}

impl<K: PrettyPrint, V: PrettyPrint> PrettyPrint for alloc::collections::BTreeMap<K, V> {
    fn render(&self) -> Document {
        let single = self.iter().fold(Document::Empty, |acc, (k, v)| match acc {
            Document::Empty => k.render() + " => " + v.render(),
            acc => acc + ',' + ' ' + k.render() + " => " + v.render(),
        });
        let multi = self.iter().fold(Document::Empty, |acc, (k, v)| match acc {
            Document::Empty => k.render() + " => " + v.render(),
            acc => acc + ',' + nl() + k.render() + " => " + v.render(),
        });
        let single_line = '{' + single + '}';
        let multi_line = '{' + indent(4, nl() + multi) + nl() + '}';
        single_line | multi_line
    }
}

struct Prettier<'a, P: ?Sized + PrettyPrint>(&'a P);

impl<'a, P: ?Sized + PrettyPrint> fmt::Display for Prettier<'a, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.pretty_print(f)
    }
}
