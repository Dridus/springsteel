#![doc(hidden)]
//! Module containing the [`constraint!`](crate::constraint) and
//! [`add_constraint!`](crate::add_constraint) macros which define a shorthand grammar for building
//! constraints and a quick way to add those to a [`gtk::ConstraintLayout`], respectively.
//!
//! Similar in concept to the VFL supported by
//! [`gtk::ConstraintLayout::add_constraints_from_description`] but instead of using a run-time
//! parsed string and a map of view names to widget instances, checked and built at compile-time.
//!
//! See [`constraint!`](crate::constraint) for a description of the grammar.

/// Translate a constraint attribute by keyword (left, right, etc.) into the corresponding
/// [`gtk::ConstraintAttribute`] value.
#[doc(hidden)]
#[macro_export]
macro_rules! constraint_attribute {
    (left) => { gtk::ConstraintAttribute::Left };
    (right) => { gtk::ConstraintAttribute::Right };
    (top) => { gtk::ConstraintAttribute::Top };
    (bottom) => { gtk::ConstraintAttribute::Bottom };
    (start) => { gtk::ConstraintAttribute::Start };
    (end) => { gtk::ConstraintAttribute::End };
    (width) => { gtk::ConstraintAttribute::Width };
    (height) => { gtk::ConstraintAttribute::Height };
    (center_x) => { gtk::ConstraintAttribute::CenterX };
    (center_y) => { gtk::ConstraintAttribute::CenterY };
}

/// Translate a strength keyword or literal value into the corresponding `i32` or if no keyword
/// provided then expand to required strength.
#[doc(hidden)]
#[macro_export]
macro_rules! constraint_strength {
    () => (gtk::ffi::GTK_CONSTRAINT_STRENGTH_REQUIRED);
    (required) => (gtk::ffi::GTK_CONSTRAINT_STRENGTH_REQUIRED);
    (strong) => (gtk::ffi::GTK_CONSTRAINT_STRENGTH_STRONG);
    (medium) => (gtk::ffi::GTK_CONSTRAINT_STRENGTH_MEDIUM);
    (weak) => (gtk::ffi::GTK_CONSTRAINT_STRENGTH_WEAK);
    ($s:literal) => ($s);
}

/// Translate a relation operator (`==`, `<=`, or `>=`) into the equivalent
/// [`gtk::ConstraintRelation`].
#[doc(hidden)]
#[macro_export]
macro_rules! constraint_relation {
    (==) => { gtk::ConstraintRelation::Eq };
    (<=) => { gtk::ConstraintRelation::Le };
    (>=) => { gtk::ConstraintRelation::Ge };
}

/// Translate some combination of positive and negative constants, both of which are optional, into
/// a final constant value for a constraint.
#[doc(hidden)]
#[macro_export]
macro_rules! constraint_constant {
    (;) => (0.0);
    ($c_p:literal; ) => ($c_p);
    (; $c_n:literal) => (0.0 - $c_n);
    ($c_p:literal; $c_n:literal) => ($c_p - $c_n);
}

/// Translate some combination of multiplier and divisor, both of which are optional, into a final
/// coefficient value for a constraint.
#[doc(hidden)]
#[macro_export]
macro_rules! constraint_multiplier {
    (;) => (1.0);
    ($f:literal; ) => ($f);
    (; $d:literal) => (1/$d);
    ($f:literal; $d:literal) => ($f * (1/$d));
}

/// Generate a [`gtk::Constraint`] from a small grammar, for brevity.
///
/// Example using [`add_constraint!`](crate::add_constraint) which uses this grammar:
///
/// ```
///    # use springsteel::{ConstraintView, add_constraint};
///    # use gtk::{Button, ConstraintGuide, Label};
///    # use gtk::prelude::WidgetExt as _;
///    # gtk::init().expect("gtk::init");
///    #
///    let display = Label::new(None);
///    let increment = Button::with_label("+");
///    let decrement = Button::with_label("-");
///
///    let content = ConstraintView::new();
///    display.set_parent(&content);
///    increment.set_parent(&content);
///    decrement.set_parent(&content);
///
///    let content_layout = content.layout();
///    let content_body = ConstraintGuide::new();
///    let controls_display_spacer = ConstraintGuide::new();
///    content_layout.add_guide(&content_body);
///    content_layout.add_guide(&controls_display_spacer);
///
///    add_constraint!(content_layout, content_body.top == top + 20.0);
///    add_constraint!(content_layout, content_body.left == left + 20.0);
///    add_constraint!(content_layout, right == content_body.right + 20.0);
///    add_constraint!(content_layout, bottom == content_body.bottom + 20.0);
///
///    add_constraint!(content_layout, increment.top == content_body.top);
///    add_constraint!(content_layout, increment.left == content_body.left);
///    add_constraint!(content_layout, increment.right == controls_display_spacer.left);
///
///    add_constraint!(content_layout, increment.width == increment.height);
///
///    add_constraint!(content_layout, decrement.top == increment.bottom + 10.0);
///
///    add_constraint!(content_layout, decrement.bottom == content_body.bottom);
///    add_constraint!(content_layout, decrement.left == content_body.left);
///    add_constraint!(content_layout, decrement.right == controls_display_spacer.left);
///
///    add_constraint!(content_layout, increment.height == decrement.height);
///
///    add_constraint!(content_layout, controls_display_spacer.width == 10.0);
///
///    add_constraint!(content_layout, display.top == content_body.top);
///    add_constraint!(content_layout, display.left == controls_display_spacer.right);
///    add_constraint!(content_layout, display.right == content_body.end);
///    add_constraint!(content_layout, display.bottom == content_body.bottom);
/// ```
///
/// Two forms are supported, a constant form:
///
/// `TARGET OP LITERAL [@STRENGTH]`
///
/// And the general form:
///
/// `TARGET OP SOURCE [* FACTOR] [/ DIVISOR] [+ CONSTANT] [- CONSTANT] [@STRENGTH]`
///
/// In either form:
///
///  - `TARGET` or `SOURCE`: either `IDENT.ATTR` or just `ATTR`. `IDENT.ATTR` means the given
///    attribute of some guide or widget within the layout, whereas `ATTR` by itself means the
///    given attribute of the widget which is being laid out, i.e. the container. Attributes are as
///    given in [`gtk::ConstraintAttribute`] but in `lower_kebab_case`: `left`, `right`, `top`,
///    `bottom`, `start`, `end`, `width`, `height`, `center_x`, and `center_y`.
///
///  - `OP`: the constraint relation, usually `==` but `<=` and `>=` can also be used.
///
///  - `LITERAL`: a literal value which is used as the constant.
///
///  - `[* FACTOR]`, `[/ DIVISOR]`, `[+ CONSTANT]`, `[- CONSTANT]`: factor and constant value
///    applied to the right hand side then related to the left hand side. E.g.
///    `width == height * 2 + 10` makes the width of the laid out widget be twice the height
///    plus 10. `/ DIVISOR` is equivalent to `* (1/DIVISOR)`, while `- CONSTANT` is equivalent to
///    `+ (-CONSTANT)`.
///
///  - `[@STRENGTH]`: optional constraint strength. If not given, defaults to required.
///    Strength can be one of the enumerated strength values given as a keyword, or a literal i32
///    strength value. Keywords supported: `required`, `strong`, `medium`, `weak`.
///
/// See also [`add_constraint!`](crate::add_constraint) which makes it even more brief to add a
/// constraint to a [`gtk::ConstraintLayout`].
#[macro_export]
macro_rules! constraint {
    (
        $lhs:ident.$lhs_attr:ident $relation:tt $lit:literal
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new_constant(
            Some(&$lhs), // target
            $crate::constraint_attribute!($lhs_attr), // target_attribute
            $crate::constraint_relation!($relation),
            $lit, // constant
            $crate::constraint_strength!($($strength)?),
        )
    );

    (
        $lhs_attr:ident $relation:tt $lit:literal
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new_constant(
            None::<&gtk::ConstraintGuide>,
            $crate::constraint_attribute!($lhs_attr),
            $crate::constraint_relation!($relation),
            $lit,
            $crate::constraint_strength!($($strength)?),
        )
    );

    (
        $lhs:ident.$lhs_attr:ident
        $relation:tt
        $rhs:ident.$rhs_attr:ident
            $(* $f:literal)? $(/ $d:literal)?
            $(+ $c_p:literal)? $(- $c_n:literal)?
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new(
            Some(&$lhs),
            $crate::constraint_attribute!($lhs_attr),
            $crate::constraint_relation!($relation),
            Some(&$rhs),
            $crate::constraint_attribute!($rhs_attr),
            $crate::constraint_multiplier!($($f)?; $($d)?),
            $crate::constraint_constant!($($c_p)?; $($c_n)?),
            $crate::constraint_strength!($($strength)?),
        )
    );

    (
        $lhs:ident.$lhs_attr:ident
        $relation:tt
        $rhs_attr:ident
            $(* $f:literal)? $(/ $d:literal)?
            $(+ $c_p:literal)? $(- $c_n:literal)?
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new(
            Some(&$lhs),
            $crate::constraint_attribute!($lhs_attr),
            $crate::constraint_relation!($relation),
            None::<&gtk::ConstraintGuide>,
            $crate::constraint_attribute!($rhs_attr),
            $crate::constraint_multiplier!($($f)?; $($d)?),
            $crate::constraint_constant!($($c_p)?; $($c_n)?),
            $crate::constraint_strength!($($strength)?),
        )
    );

    (
        $lhs_attr:ident
        $relation:tt
        $rhs:ident.$rhs_attr:ident
            $(* $f:literal)? $(/ $d:literal)?
            $(+ $c_p:literal)? $(- $c_n:literal)?
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new(
            None::<&gtk::ConstraintGuide>,
            $crate::constraint_attribute!($lhs_attr),
            $crate::constraint_relation!($relation),
            Some(&$rhs),
            $crate::constraint_attribute!($rhs_attr),
            $crate::constraint_multiplier!($($f)?; $($d)?),
            $crate::constraint_constant!($($c_p)?; $($c_n)?),
            $crate::constraint_strength!($($strength)?),
        )
    );

    (
        $lhs_attr:ident
        $relation:tt
        $rhs_attr:ident
            $(* $f:literal)? $(/ $d:literal)?
            $(+ $c_p:literal)? $(- $c_n:literal)?
        $(@$strength:tt)?
    ) => (
        gtk::Constraint::new(
            None::<&gtk::ConstraintGuide>,
            $crate::constraint_attribute!($lhs_attr),
            $crate::constraint_relation!($relation),
            None::<&gtk::ConstraintGuide>,
            $crate::constraint_attribute!($rhs_attr),
            $crate::constraint_multiplier!($($f)?; $($d)?),
            $crate::constraint_constant!($($c_p)?; $($c_n)?),
            $crate::constraint_strength!($($strength)?),
        )
    );
}

/// Create a [`gtk::Constraint`] using the grammar of [`constraint!`] and then add it to a given
/// [`gtk::ConstraintLayout`].
///
/// E.g.
/// ```
///    # use springsteel::add_constraint;
///    # gtk::init().expect("gtk::init");
///    # let content_layout = gtk::ConstraintLayout::new();
///    # let increment = gtk::Button::with_label("+");
///    # let decrement = gtk::Button::with_label("-");
///    #
///    add_constraint!(content_layout, increment.height == decrement.height);
/// ```
#[macro_export]
macro_rules! add_constraint {
    ($layout:expr, $($constraint:tt)*) => {
        $layout.add_constraint(&$crate::constraint!($($constraint)*));
    };
}
