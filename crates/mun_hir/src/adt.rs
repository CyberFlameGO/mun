use std::{fmt, sync::Arc};

use crate::type_ref::{LocalTypeRefId, TypeRefBuilder, TypeRefMap, TypeRefSourceMap};
use crate::{
    arena::{Arena, Idx},
    ids::{StructId, TypeAliasId},
    AsName, DefDatabase, Name,
};
use mun_syntax::ast::{self, NameOwner, TypeAscriptionOwner};

pub use mun_syntax::ast::StructMemoryKind;

use crate::ids::Lookup;

/// A single field of a record
/// ```mun
/// struct Foo {
///     a: int, // <- this
/// }
/// ```
/// or
/// ```mun
/// struct Foo(
///     int, // <- this
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldData {
    pub name: Name,
    pub type_ref: LocalTypeRefId,
}

/// A struct's fields' data (record, tuple, or unit struct)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructKind {
    Record,
    Tuple,
    Unit,
}

impl fmt::Display for StructKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StructKind::Record => write!(f, "record"),
            StructKind::Tuple => write!(f, "tuple"),
            StructKind::Unit => write!(f, "unit struct"),
        }
    }
}

/// An identifier for a struct's or tuple's field
pub type LocalStructFieldId = Idx<StructFieldData>;

#[derive(Debug, PartialEq, Eq)]
pub struct StructData {
    pub name: Name,
    pub fields: Arena<StructFieldData>,
    pub kind: StructKind,
    pub memory_kind: StructMemoryKind,
    type_ref_map: TypeRefMap,
    type_ref_source_map: TypeRefSourceMap,
}

impl StructData {
    pub(crate) fn struct_data_query(db: &dyn DefDatabase, id: StructId) -> Arc<StructData> {
        let loc = id.lookup(db);
        let item_tree = db.item_tree(loc.id.file_id);
        let strukt = &item_tree[loc.id.value];
        let src = item_tree.source(db, loc.id);

        let memory_kind = src
            .memory_type_specifier()
            .map(|s| s.kind())
            .unwrap_or_default();

        let mut type_ref_builder = TypeRefBuilder::default();
        let (fields, kind) = match src.kind() {
            ast::StructKind::Record(r) => {
                let fields = r
                    .fields()
                    .map(|fd| StructFieldData {
                        name: fd.name().map(|n| n.as_name()).unwrap_or_else(Name::missing),
                        type_ref: type_ref_builder.alloc_from_node_opt(fd.ascribed_type().as_ref()),
                    })
                    .collect();
                (fields, StructKind::Record)
            }
            ast::StructKind::Tuple(t) => {
                let fields = t
                    .fields()
                    .enumerate()
                    .map(|(index, fd)| StructFieldData {
                        name: Name::new_tuple_field(index),
                        type_ref: type_ref_builder.alloc_from_node_opt(fd.type_ref().as_ref()),
                    })
                    .collect();
                (fields, StructKind::Tuple)
            }
            ast::StructKind::Unit => (Arena::default(), StructKind::Unit),
        };

        let (type_ref_map, type_ref_source_map) = type_ref_builder.finish();
        Arc::new(StructData {
            name: strukt.name.clone(),
            fields,
            kind,
            memory_kind,
            type_ref_map,
            type_ref_source_map,
        })
    }

    pub fn type_ref_source_map(&self) -> &TypeRefSourceMap {
        &self.type_ref_source_map
    }

    pub fn type_ref_map(&self) -> &TypeRefMap {
        &self.type_ref_map
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeAliasData {
    pub name: Name,
    pub type_ref_id: LocalTypeRefId,
    type_ref_map: TypeRefMap,
    type_ref_source_map: TypeRefSourceMap,
}
impl TypeAliasData {
    pub(crate) fn type_alias_data_query(
        db: &dyn DefDatabase,
        id: TypeAliasId,
    ) -> Arc<TypeAliasData> {
        let loc = id.lookup(db);
        let item_tree = db.item_tree(loc.id.file_id);
        let alias = &item_tree[loc.id.value];
        let src = item_tree.source(db, loc.id);
        let mut type_ref_builder = TypeRefBuilder::default();
        let type_ref_opt = src.type_ref();
        let type_ref_id = type_ref_builder.alloc_from_node_opt(type_ref_opt.as_ref());
        let (type_ref_map, type_ref_source_map) = type_ref_builder.finish();
        Arc::new(TypeAliasData {
            name: alias.name.clone(),
            type_ref_id,
            type_ref_map,
            type_ref_source_map,
        })
    }

    pub fn type_ref_source_map(&self) -> &TypeRefSourceMap {
        &self.type_ref_source_map
    }

    pub fn type_ref_map(&self) -> &TypeRefMap {
        &self.type_ref_map
    }
}
