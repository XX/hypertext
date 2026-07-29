mod buffer;
mod impls;

use core::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

pub use self::buffer::*;
use crate::{
    Raw, Rendered,
    alloc::string::String,
    const_precise_live_drops_hack,
    context::{AttributeValue, Attributes, Context, Node},
};

/// A type that can be rendered as an HTML node.
///
/// For [`Renderable<Node>`] (a.k.a. [`Renderable`]) implementations, this
/// must render complete HTML nodes. If rendering string-like types, the
/// implementation must escape `&` to `&amp;`, `<` to `&lt;`, and `>` to `&gt;`.
///
/// For [`Renderable<AttributeValue>`] implementations, this must render an
/// attribute value which will eventually be surrounded by double quotes. The
/// implementation must escape `&` to `&amp;`, `<` to `&lt;`, `>` to `&gt;`, and
/// `"` to `&quot;`.
///
/// For [`Renderable<Attributes>`] implementations, this must render zero or
/// more attributes as written inside an element's opening tag, each preceded by
/// a space. Such values are rendered by the `(...)` spread syntax in attribute
/// position, and should be built using [`AttributesBuffer::push_attribute`]
/// and [`AttributesBuffer::push_empty_attribute`]:
///
/// ```
/// use hypertext::{AttributesBuffer, context::Attributes, prelude::*};
///
/// pub struct DataAttrs(Vec<(String, String)>);
///
/// impl Renderable<Attributes> for DataAttrs {
///     fn render_to(&self, buffer: &mut AttributesBuffer) {
///         for (name, value) in &self.0 {
///             buffer.push_attribute(name, value);
///         }
///     }
/// }
///
/// let attrs = DataAttrs(vec![("data-id".into(), "42".into())]);
///
/// assert_eq!(
///     maud! { div (attrs) { "content" } }.render().as_inner(),
///     r#"<div data-id="42">content</div>"#,
/// );
/// ```
///
/// # Examples
///
/// ## Implementing [`Renderable`]
///
/// There are 3 ways to implement this trait.
///
/// ### Manual [`impl Renderable`](Renderable)
///
/// ```
/// use hypertext::{Buffer, prelude::*};
///
/// pub struct Person {
///     name: String,
///     age: u8,
/// }
///
/// impl Renderable for Person {
///     fn render_to(&self, buffer: &mut Buffer) {
///         buffer.push(maud! {
///             div {
///                 h1 { (self.name) }
///                 p { "Age: " (self.age) }
///             }
///         });
///     }
/// }
///
/// let person = Person {
///     name: "Alice".into(),
///     age: 20,
/// };
///
/// assert_eq!(
///     maud! { main { (person) } }.render().as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p></div></main>",
/// );
/// ```
///
/// ### [`#[derive(Renderable)]`](derive@crate::Renderable)
///
/// ```
/// use hypertext::prelude::*;
///
/// #[derive(Renderable)]
/// #[maud(
///     div {
///         h1 { (self.name) }
///         p { "Age: " (self.age) }
///     }
/// )]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let person = Person {
///     name: "Alice".into(),
///     age: 20,
/// };
///
/// assert_eq!(
///     maud! { main { (person) } }.render().as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p></div></main>",
/// );
/// ```
///
/// ### [`#[renderable]`](crate::renderable)
///
/// ```
/// use hypertext::prelude::*;
/// #[renderable]
/// fn person(name: &String, age: u8) -> impl Renderable {
///     maud! {
///         div {
///             h1 { (name) }
///             p { "Age: " (age) }
///         }
///     }
/// }
///
/// assert_eq!(
///     maud! { main { (Person { name: "Alice".into(), age: 20 }) } }
///         .render()
///         .as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p></div></main>",
/// );
/// ```
///
/// ## Component Syntax
///
/// In addition to the standard way of rendering a [`Renderable`] struct inside
/// a `(...)` node, you can also use the "component" syntax to make using these
/// types more like popular frontend frameworks such as React.js.
///
/// ### [`maud!`](crate::maud!)
///
/// ```
/// use hypertext::prelude::*;
///
/// #[renderable]
/// fn person(name: &String, age: u8) -> impl Renderable {
///     maud! {
///         div {
///             h1 { (name) }
///             p { "Age: " (age) }
///         }
///     }
/// }
///
/// assert_eq!(
///     maud! { main { Person name=("Alice".into()) age=20; } }
///         .render()
///         .as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p></div></main>",
/// );
/// ```
///
/// ### [`rsx!`](crate::rsx!)
///
/// ```
/// use hypertext::prelude::*;
///
/// #[renderable]
/// fn person(name: &String, age: u8) -> impl Renderable {
///     rsx! {
///         <div>
///             <h1>(name)</h1>
///             <p>"Age: " (age)</p>
///         </div>
///     }
/// }
///
/// assert_eq!(
///     rsx! {
///         <main>
///             <Person name=("Alice".into()) age=20>
///         </main>
///     }
///     .render()
///     .as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p></div></main>",
/// );
/// ```
///
/// ### `children`
///
/// If you add children to the component node, the macro will pass them to the
/// `children` field of the type as a [`Lazy`].
///
/// ```
/// use hypertext::prelude::*;
///
/// #[renderable]
/// fn person<R: Renderable>(name: &String, age: u8, children: &R) -> impl Renderable {
///     maud! {
///         div {
///             h1 { (name) }
///             p { "Age: " (age) }
///             (children)
///         }
///     }
/// }
///
/// assert_eq!(
///     maud! {
///         main {
///             Person name=("Alice".into()) age=20 {
///                 p { "Pronouns: she/her" }
///             }
///         }
///     }
///     .render()
///     .as_inner(),
///     "<main><div><h1>Alice</h1><p>Age: 20</p><p>Pronouns: she/her</p></div></main>",
/// );
/// ```
pub trait Renderable<C: Context = Node> {
    /// Renders this value to the buffer.
    fn render_to(&self, buffer: &mut Buffer<C>);

    /// Creates a new [`Buffer<C>`] from this value.
    ///
    /// This is a convenience method that creates a new [`Buffer<C>`],
    /// [`push`](Buffer::push)es `self` to it, then returns it.
    ///
    /// This may be overridden if `Self` is a string-like pre-escaped type that
    /// can more efficiently be turned into a [`Buffer<C>`] via
    /// [`Buffer::dangerously_from_string`]. If overridden, the
    /// implementation must match what [`render_to`](Renderable::render_to)
    /// would produce.
    #[inline]
    fn to_buffer(&self) -> Buffer<C> {
        let mut buffer = Buffer::<C>::new();
        buffer.push(self);
        buffer
    }
}

/// An extension trait for [`Renderable`] types.
///
/// This trait provides an additional method for rendering and memoizing values.
pub trait RenderableExt: Renderable {
    /// Renders this value to a [`Rendered<String>`].
    ///
    /// This is usually the final step in rendering a value, converting it
    /// into a [`Rendered<String>`](Rendered) that can be returned as an HTTP
    /// response or written to a file.
    #[inline]
    fn render(&self) -> Rendered<String> {
        self.to_buffer().rendered()
    }

    /// Pre-renders the value and stores it in a [`Raw`] so that it can be
    /// re-used among multiple renderings without re-computing the value.
    ///
    /// This should generally be avoided to prevent unnecessary allocations, but
    /// may be useful if it is more expensive to compute and render the value.
    #[inline]
    fn memoize(&self) -> Raw<String> {
        // XSS SAFETY: The value has already been rendered and is assumed as safe.
        Raw::dangerously_create(self.to_buffer().into_inner())
    }
}

impl<T: Renderable> RenderableExt for T {}

/// A value lazily rendered via a closure.
///
/// For [`Lazy<F, Node>`] (a.k.a. [`Lazy<F>`]), this must render complete
/// HTML nodes. If rendering string-like types, the closure must escape `&` to
/// `&amp;`, `<` to `&lt;`, and `>` to `&gt;`.
///
/// For [`Lazy<F, AttributeValue>`] (a.k.a. [`LazyAttribute<F>`]), this must
/// render an attribute value which will eventually be surrounded by double
/// quotes. The closure must escape `&` to `&amp;`, `<` to `&lt;`, `>` to
/// `&gt;`, and `"` to `&quot;`.
///
/// For [`Lazy<F, Attributes>`] (a.k.a. [`LazyAttributes<F>`]), this must render
/// zero or more attributes as written inside an element's opening tag, each
/// preceded by a space.
#[derive(Clone, Copy)]
#[must_use = "`Lazy` does nothing unless `.render()` or `.render_to()` is called"]
pub struct Lazy<F: Fn(&mut Buffer<C>), C: Context = Node> {
    f: F,
    context: PhantomData<C>,
}

/// An attribute value lazily rendered via a closure.
///
/// This is a type alias for [`Lazy<F, AttributeValue>`].
pub type LazyAttribute<F> = Lazy<F, AttributeValue>;

/// A list of attributes lazily rendered via a closure.
///
/// This is a type alias for [`Lazy<F, Attributes>`].
pub type LazyAttributes<F> = Lazy<F, Attributes>;

impl<F: Fn(&mut Buffer<C>), C: Context> Lazy<F, C> {
    /// Creates a new [`Lazy`] from the given closure.
    ///
    /// It is recommended to add a `// XSS SAFETY` comment above the usage of
    /// this function to indicate why it is safe to assume that the closure will
    /// not write possibly unsafe HTML to the buffer.
    #[inline]
    pub const fn dangerously_create(f: F) -> Self {
        Self {
            f,
            context: PhantomData,
        }
    }

    /// Extracts the inner closure.
    #[inline]
    pub const fn into_inner(self) -> F {
        // SAFETY: `Lazy<F, C>` has exactly one non-zero-sized field, which is `f`.
        unsafe { const_precise_live_drops_hack!(self.f) }
    }

    /// Gets a reference to the inner closure.
    #[inline]
    pub const fn as_inner(&self) -> &F {
        &self.f
    }
}

impl<C: Context> Default for Lazy<fn(&mut Buffer<C>), C> {
    #[inline]
    fn default() -> Self {
        Self::dangerously_create(|_| ())
    }
}

impl<F: Fn(&mut Buffer<C>), C: Context> Renderable<C> for Lazy<F, C> {
    #[inline]
    fn render_to(&self, buffer: &mut Buffer<C>) {
        (self.f)(buffer);
    }
}

impl<F: Fn(&mut Buffer<C>), C: Context> Debug for Lazy<F, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Lazy").finish_non_exhaustive()
    }
}

/// A single attribute whose name is only known at runtime.
///
/// This is the building block for spreading a collection of attributes into an
/// element via the `(...)` syntax of [`maud!`](crate::maud!) and
/// [`rsx!`](crate::rsx!), as [`Renderable<Attributes>`] is implemented for
/// slices, arrays and [`Vec`](crate::alloc::vec::Vec)s of it, as well as for
/// [`Option`] and tuples.
///
/// Unlike attributes written directly in the macros, the name is not checked
/// against the element it is rendered into, and so it must be a valid attribute
/// name as reported by [`AttributesBuffer::is_valid_attribute_name`].
///
/// # Example
///
/// ```
/// use hypertext::{NamedAttribute, prelude::*};
///
/// let attrs = vec![
///     NamedAttribute::new("data-id", "42"),
///     NamedAttribute::new("data-label", "Answer"),
/// ];
///
/// assert_eq!(
///     maud! { div.card (attrs) { "content" } }.render().as_inner(),
///     r#"<div class="card" data-id="42" data-label="Answer">content</div>"#,
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamedAttribute<N: AsRef<str>, V: Renderable<AttributeValue>> {
    name: N,
    value: Option<V>,
}

impl<N: AsRef<str>, V: Renderable<AttributeValue>> NamedAttribute<N, V> {
    /// Creates an attribute with the given name and value.
    #[inline]
    pub const fn new(name: N, value: V) -> Self {
        Self {
            name,
            value: Some(value),
        }
    }

    /// Gets the name of the attribute.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Gets the value of the attribute, if it has one.
    #[inline]
    pub const fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    /// Creates a valueless (boolean) attribute with the given name, rendered
    /// without a `="..."` part.
    ///
    /// # Example
    ///
    /// ```
    /// use hypertext::{NamedAttribute, prelude::*};
    ///
    /// let attrs = [NamedAttribute::<_, ()>::empty("disabled")];
    ///
    /// assert_eq!(
    ///     rsx! { <input type="checkbox" (attrs) /> }
    ///         .render()
    ///         .as_inner(),
    ///     r#"<input type="checkbox" disabled>"#,
    /// );
    /// ```
    #[inline]
    pub const fn empty(name: N) -> Self {
        Self { name, value: None }
    }
}

impl<N: AsRef<str>, V: Renderable<AttributeValue>> Renderable<Attributes> for NamedAttribute<N, V> {
    #[inline]
    fn render_to(&self, buffer: &mut AttributesBuffer) {
        if let Some(value) = &self.value {
            buffer.push_attribute(self.name.as_ref(), value);
        } else {
            buffer.push_empty_attribute(self.name.as_ref());
        }
    }
}

/// A value rendered via its [`Display`] implementation.
///
/// This will handle escaping special characters for you.
///
/// This can be created more easily via the `%(...)` syntax in
/// [`maud!`](crate::maud!), [`rsx!`](crate::rsx!), and
/// [`attribute!`](crate::attribute!) which will automatically wrap the
/// expression in this type.
///
/// # Example
///
/// ```
/// use std::fmt::{self, Display, Formatter};
///
/// use hypertext::prelude::*;
///
/// struct Greeting(&'static str);
///
/// impl Display for Greeting {
///     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
///         write!(f, "Hello, {}! <script>", self.0)
///     }
/// }
///
/// assert_eq!(
///     maud! { div { %(Greeting("Alice")) } }.render().as_inner(),
///     "<div>Hello, Alice! &lt;script&gt;</div>",
/// );
/// ```
#[derive(Debug, Clone, Copy)]
#[doc(alias = "%(...)")]
pub struct Displayed<T: Display>(pub T);

impl<C: Context, T: Display> Renderable<C> for Displayed<T>
where
    for<'a> fmt::Arguments<'a>: Renderable<C>,
{
    #[inline]
    fn render_to(&self, buffer: &mut Buffer<C>) {
        format_args!("{}", self.0).render_to(buffer);
    }
}

/// A value rendered via its [`Debug`] implementation.
///
/// This will handle escaping special characters for you.
///
/// This can be created more easily via the `?(...)` syntax in
/// [`maud!`](crate::maud!), [`rsx!`](crate::rsx!), and
/// [`attribute!`](crate::attribute!) which will automatically wrap the
/// expression in this type.
///
/// # Example
///
/// ```
/// use hypertext::prelude::*;
///
/// #[derive(Debug)]
/// struct Greeting(&'static str);
///
/// assert_eq!(
///     maud! {
///         div title=?(Greeting("Alice")) {
///             ?(Greeting("Alice"))
///         }
///     }
///     .render()
///     .as_inner(),
///     r#"<div title="Greeting(&quot;Alice&quot;)">Greeting("Alice")</div>"#,
/// );
/// ```
#[derive(Debug, Clone, Copy)]
#[doc(alias = "?(...)")]
pub struct Debugged<T: Debug>(pub T);

impl<C: Context, T: Debug> Renderable<C> for Debugged<T>
where
    for<'a> fmt::Arguments<'a>: Renderable<C>,
{
    #[inline]
    fn render_to(&self, buffer: &mut Buffer<C>) {
        format_args!("{:?}", self.0).render_to(buffer);
    }
}
