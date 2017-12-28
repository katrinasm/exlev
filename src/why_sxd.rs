//! SXD is full of awful choices. Some of them seem Java-y but some of them
//! seem like regular bad design.
//! But it's the only XPath library in Rust, so I wrote this module to hopefully
//! make it bearable.

use sxd_xpath::{self, XPath, Context, Error, Value};
use sxd_xpath::nodeset::{Nodeset, Node};

/// Make an XPath without exposing the intermediate Factory.
///
/// For some reason, SXD uses a factory.
/// Since `Factory::new` doesn't do a lot but `Factory::build` does,
/// there doesn't seem to be much reason to have not just made an `XPath::new`.
/// Also, the return type of `Factory::build` is `Result<Option<XPath>, Error>`.
/// I am not sure when a Factory could make no XPath and have that NOT be an error.
/// `make_xpath` and `make_xpath_static` are convenience functions to reduce this
/// weird boilerplate.
/// `make_xpath` just makes a factory, then builds an `XPath`. If building
/// the XPath would return `None`, `make_xpath` returns `Error::NoXPath`.
///
/// # Examples
/// ```rust
/// extern crate sxd_document;
/// extern crate sxd_xpath;
/// use sxd_document::dom::Document;
/// use sxd_xpath::{Value, Error, Context};
/// fn foos_with_name<'d>(doc: &'d Document, name: &str) -> Result<Value<'d>, Error> {
///     let ctx = Context::new();
///     let xp = make_xpath(&format!("/foo[@name='{}']", name))?;
///     xp.evaluate(&ctx, doc.root()).map_err(Error::from)
/// }
/// ```

pub fn make_xpath(path: &str) -> Result<XPath, Error> {
    sxd_xpath::Factory::new().build(path)?.ok_or(Error::NoXPath)
}

/// Make an XPath from a static string,
/// without exposing the intermediate Factory.
///
/// Makes an XPath from a string known at compile-time, and if any error occurs,
/// panics. It is up to the caller to avoid this panic.
/// `make_xpath_static` is more convenient than `make_xpath` when the paths
/// given to the function can be tested exhaustively.
/// An example of this is passing the function a single string literal.
/// It could also be chosen from a small array of strs, or something.
///
/// # Examples
/// ```rust
/// extern crate sxd_document;
/// extern crate sxd_xpath;
/// use sxd_document::dom::Document;
/// use sxd_xpath::{Value, Error, Context};
/// fn get_cows<'d>(doc: &'d Document, name: &str) -> Result<Value<'d>, Error> {
///     let ctx = Context::new();
///     // Note that there isn't any question mark after this line
///     let xp = make_xpath_static("/animals/animal[@sound='moo']");
///     xp.evaluate(&ctx, doc.root()).map_err(Error::from)
/// }
/// ```
pub fn make_xpath_static(path: &'static str) -> XPath {
    sxd_xpath::Factory::new().build(path).unwrap().unwrap()
}

/// Evaluates an `XPath`, forcing the result to be a `Nodeset`.
pub fn xpath_nodes<'d, N>(
    context: &Context<'d>,
    node: N,
    path: &XPath,
) -> Result<Nodeset<'d>, Error>
where
    N: Into<Node<'d>>,
{
    let v = path.evaluate(context, node)?;
    if let Value::Nodeset(ns) = v {
        Ok(ns)
    } else {
        Ok(Nodeset::new())
    }
}

/// Converts `path` to an `XPath`,
/// then evaluates it, forcing the result to be a `Nodeset`.
pub fn xpath_nodes_str<'d, N>(
    context: &Context<'d>,
    node: N,
    path: &str,
) -> Result<Nodeset<'d>, Error>
where
    N: Into<Node<'d>>,
{
    let xp = make_xpath(path)?;
    xpath_nodes(context, node, &xp)
}

pub fn node_element_attr<'n>(node: &'n Node, name: &str) -> Option<&'n str> {
    node.element().and_then(|el| el.attribute(name)).and_then(
        |att| {
            Some(att.value())
        },
    )
}

pub fn attr_hexbyte(node: &Node, name: &str) -> Option<u8> {
    node_element_attr(node, name).and_then(|v| u8::from_str_radix(v, 16).ok())
}

pub fn attr_u32(node: &Node, name: &str) -> Option<u32> {
    node_element_attr(node, name)
        .and_then(|v| v.parse::<f64>().ok())
        .map(|f| f as u32)
}

pub fn attr_bool(node: &Node, name: &str) -> Option<bool> {
    node_element_attr(node, name)
        .and_then(|v| v.parse::<bool>().ok())
}

pub fn only_match<'d, N>(context: &Context<'d>, node: N, path: &XPath) -> Option<Node<'d>>
where
    N: Into<Node<'d>>,
{
    let ns = if let Ok(v) = xpath_nodes(context, node, path) {
        v
    } else {
        return None;
    };
    if ns.size() > 1 {
        None
    } else {
        ns.document_order_first()
    }
}

pub fn only_match_str<'d, N>(context: &Context<'d>, node: N, path: &'static str) -> Option<Node<'d>>
where
    N: Into<Node<'d>>,
{
    let xp = make_xpath_static(path);
    only_match(context, node, &xp)
}
