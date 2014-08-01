/*!
A Rust syntax extension for applying the `pub` visibility modifer to many items at once

Right now the attribute applies to every possible child AST element that could have
public visibility, including:

- `use`
- `static`
- `fn`, both standalone and methods/associated ones
- `mod`
- `type`, `struct` and `enum`
- `trait`
- symbols in `extern {}` blocks.

# Example

To load the extension and use it:

```rust
#![feature(phase)]

#[phase(plugin)]
extern crate apply_pub = "apply-pub-rs";

#[apply_pub]
mod foo {
    fn bar() {}
    mod baz {
        fn qux() {}
    }
}

fn main() {
    foo::bar();
    foo::baz::qux();
}
```

*/

#![crate_name = "apply-pub-rs"]
#![crate_type = "dylib"]
#![license = "MIT"]
#![feature(plugin_registrar, managed_boxes)]
#![feature(phase)]

extern crate syntax;
extern crate rustc;

use syntax::ast::ViewItem;
use syntax::ast::{MetaItem, Item, StructField, ViewItemUse, ForeignItem};
use syntax::ast::{Method, MethDecl, ItemMac, UnnamedField, NamedField};
use syntax::ast::{Public, ItemFn, ItemImpl, ItemForeignMod, ViewItemExternCrate};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, ItemModifier};
use syntax::fold::Folder;
use syntax::fold;
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;

use rustc::plugin::Registry;

use std::gc::{Gc, GC};

static NAME: &'static str = "apply_pub";

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern(NAME), ItemModifier(expand));
}

struct ApplyPubFolder {
    parent_item_variant: ItemVariant
}

fn expand(ctxt: &mut ExtCtxt, span: Span, _meta: Gc<MetaItem>, item: Gc<Item>) -> Gc<Item> {
    match item.node {
        ItemMac(..) => {
            ctxt.span_err(span,
                format!("Can not apply `#[{}]` to a macro invocation", NAME).as_slice());
            item
        }
        _ => {
            let expanded = ctxt.expander().fold_item(item).move_iter().nth(0).unwrap();
            let mut folder = ApplyPubFolder { parent_item_variant: IsOther };
            folder.fold_item(expanded).move_iter().nth(0).unwrap()
        }
    }
}

enum ItemVariant {
    IsFn,
    IsTraitImpl,
    IsTypeImpl,
    IsExternBlock,
    IsOther,
}

impl ItemVariant {
    fn is_fn(self) -> bool { match self { IsFn => true, _ => false } }
    fn is_trait_impl(self) -> bool { match self { IsTraitImpl => true, _ => false } }
    fn is_type_impl(self) -> bool { match self { IsTypeImpl => true, _ => false } }
    fn is_extern_block(self) -> bool { match self { IsExternBlock => true, _ => false } }
    fn can_be_made_pub(self) -> bool {
        !self.is_trait_impl() && !self.is_type_impl() && !self.is_extern_block()
    }
}

impl Folder for ApplyPubFolder {
    fn fold_view_item(&mut self, vi: &ViewItem) -> ViewItem {
        let item = fold::noop_fold_view_item(vi, self);
        match item.node {
            ViewItemExternCrate(..) => item,
            ViewItemUse(..) => ViewItem { vis: Public, ..item },
        }
    }

    fn fold_item(&mut self, i: Gc<Item>) -> SmallVector<Gc<Item>> {
        let parent_item_variant = self.parent_item_variant;

        let item_variant = match i.node {
            ItemFn(..) => IsFn,
            ItemImpl(_, None, _, _) => IsTypeImpl,
            ItemImpl(_, _, _, _) => IsTraitImpl,
            ItemForeignMod(_) => IsExternBlock,
            _ => IsOther,
        };

        self.parent_item_variant = item_variant;
        let items = fold::noop_fold_item(&*i, self);
        self.parent_item_variant = parent_item_variant;

        items.move_iter().map(|item| {
            let mut item = (*item).clone();
            if !parent_item_variant.is_fn() && item_variant.can_be_made_pub() {
                item.vis = Public;
            }
            box(GC) item
        }).collect()
    }

    fn fold_struct_field(&mut self, sf: &StructField) -> StructField {
        let mut field = fold::noop_fold_struct_field(sf, self);
        match field.node.kind {
            NamedField(_, ref mut vis) => *vis = Public,
            UnnamedField(ref mut vis) => *vis = Public,
        }
        field
    }

    fn fold_method(&mut self, m: Gc<Method>) -> SmallVector<Gc<Method>>  {
        let methods = fold::noop_fold_method(&*m, self);
        if !self.parent_item_variant.is_trait_impl() {
            methods.move_iter().map(|method| {
                let mut method = (*method).clone();
                match method.node {
                    MethDecl(_, _, _, _, _, _, _, ref mut vis) => *vis = Public,
                    _ => (),
                }
                box(GC) method
            }).collect()
        } else {
            methods
        }
    }

    fn fold_foreign_item(&mut self, ni: Gc<ForeignItem>) -> Gc<ForeignItem> {
        let foreign_item = fold::noop_fold_foreign_item(&*ni, self);
        let mut foreign_item = (*foreign_item).clone();
        foreign_item.vis = Public;
        box (GC) foreign_item
    }
}
