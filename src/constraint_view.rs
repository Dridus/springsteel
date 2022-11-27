//! Provides [`ConstraintView`], a [`gtk::Widget`] which contains other widgets and lays them out
//! using [`gtk::ConstraintLayout`].

mod imp {
    use glib::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt as _};
    use gtk::prelude::WidgetExt as _;
    use gtk::subclass::prelude::{WidgetClassSubclassExt, WidgetImpl};

    #[derive(Default)]
    pub struct ConstraintView;

    #[glib::object_subclass]
    impl ObjectSubclass for ConstraintView {
        const NAME: &'static str = "SpringsteelWorkbenchConstraintView";
        type Type = super::ConstraintView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::ConstraintLayout>();
        }
    }

    impl ObjectImpl for ConstraintView {
        fn dispose(&self) {
            let obj = self.obj();

            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ConstraintView {}
}

use glib::{Cast, Object};
use gtk::prelude::WidgetExt as _;

glib::wrapper! {
    /// [`gtk::Widget`] container (like [`gtk::Box`] or [`gtk::Grid`]) which lays out its children
    /// using a [`gtk::ConstraintLayout`].
    pub struct ConstraintView(ObjectSubclass<imp::ConstraintView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ConstraintView {
    /// Create a new empty [`ConstraintView`].
    pub fn new() -> Self {
        Object::new(&[])
    }

    /// Return the [`gtk::ConstraintLayout`] for this view.
    pub fn layout(&self) -> gtk::ConstraintLayout {
        unsafe {
            self.layout_manager()
                .expect("ConstraintView::layout_manager")
                .unsafe_cast()
        }
    }
}
