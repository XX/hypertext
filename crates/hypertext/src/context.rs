//! The [`Context`] trait and its implementors.

/// A marker trait to represent the context that the value is being rendered to.
///
/// This can be [`Node`], [`AttributeValue`], or [`Attributes`]. A [`Node`]
/// represents an HTML node, an [`AttributeValue`] represents an attribute
/// value which will eventually be surrounded by double quotes, and
/// [`Attributes`] represents a list of attributes inside an element's opening
/// tag.
///
/// This is used to ensure that the correct rendering methods are called
/// for each context, and to prevent errors such as accidentally rendering
/// an HTML element into an attribute value.
pub trait Context: sealed::Sealed {}

/// A marker trait for contexts which hold a value, as opposed to markup
/// structure.
///
/// This is implemented for [`Node`] and [`AttributeValue`], and is used to
/// restrict value-like [`Renderable`](crate::Renderable) implementations (such
/// as the ones for integers) to contexts where a bare value makes sense.
/// Notably, [`Attributes`] is *not* a value context, as an attribute list is
/// made up of name/value pairs rather than a single value.
pub trait ValueContext: Context {}

/// A marker type to represent a complete element node.
///
/// All types and traits that are generic over [`Context`] use [`Node`]
/// as the default for the generic type parameter.
///
/// Traits and types with this marker type expect complete HTML nodes. If
/// rendering string-like types, the value/implementation must escape `&` to
/// `&amp;`, `<` to `&lt;`, and `>` to `&gt;`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Node;

impl Context for Node {}

impl ValueContext for Node {}

/// A marker type to represent an attribute value.
///
/// Traits and types with this marker type expect an attribute value which will
/// eventually be surrounded by double quotes. The value/implementation must
/// escape `&` to `&amp;`, `<` to `&lt;`, `>` to `&gt;`, and `"` to `&quot;`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct AttributeValue;

impl Context for AttributeValue {}

impl ValueContext for AttributeValue {}

/// A marker type to represent a list of attributes within an element's opening
/// tag.
///
/// Traits and types with this marker type expect zero or more attributes, each
/// preceded by a space, as written inside an opening tag. Implementations
/// should render attributes using
/// [`AttributesBuffer::push_attribute`](crate::AttributesBuffer::push_attribute)
/// and
/// [`AttributesBuffer::push_empty_attribute`](crate::AttributesBuffer::push_empty_attribute),
/// which take care of escaping the value and rejecting attribute names that
/// could break out of the tag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Attributes;

impl Context for Attributes {}

mod sealed {
    use super::{AttributeValue, Attributes, Node};

    pub trait Sealed {}
    impl Sealed for Node {}
    impl Sealed for AttributeValue {}
    impl Sealed for Attributes {}
}
