use crate::code_model::src::HasSource;
use crate::diagnostics::{ExternCannotHaveBody, ExternNonPrimitiveParam};
use crate::expr::BodySourceMap;
use crate::in_file::InFile;
use crate::{diagnostics::DiagnosticSink, Body, Expr, Function, HirDatabase, InferenceResult};
use mun_syntax::{AstNode, SyntaxNodePtr};
use std::sync::Arc;

mod literal_out_of_range;
mod uninitialized_access;

#[cfg(test)]
mod tests;

pub struct ExprValidator<'a> {
    func: Function,
    infer: Arc<InferenceResult>,
    body: Arc<Body>,
    body_source_map: Arc<BodySourceMap>,
    db: &'a dyn HirDatabase,
}

impl<'a> ExprValidator<'a> {
    pub fn new(func: Function, db: &'a dyn HirDatabase) -> Self {
        let (body, body_source_map) = db.body_with_source_map(func.into());
        ExprValidator {
            func,
            db,
            infer: db.infer(func.into()),
            body,
            body_source_map,
        }
    }

    pub fn validate_body(&self, sink: &mut DiagnosticSink) {
        self.validate_literal_ranges(sink);
        self.validate_uninitialized_access(sink);
        self.validate_extern(sink);
    }

    pub fn validate_extern(&self, sink: &mut DiagnosticSink) {
        if !self.func.is_extern(self.db) {
            return;
        }

        // Validate that there is no body
        match self.body[self.func.body(self.db).body_expr] {
            Expr::Missing => {}
            _ => sink.push(ExternCannotHaveBody {
                func: self
                    .func
                    .source(self.db.upcast())
                    .map(|f| SyntaxNodePtr::new(f.syntax())),
            }),
        }

        if let Some(sig) = self.func.ty(self.db).callable_sig(self.db) {
            let fn_data = self.func.data(self.db);
            for (arg_ty, ty_ref) in sig.params().iter().zip(fn_data.params()) {
                if arg_ty.as_struct().is_some() {
                    let arg_ptr = fn_data
                        .type_ref_source_map()
                        .type_ref_syntax(*ty_ref)
                        .map(|ptr| ptr.syntax_node_ptr())
                        .unwrap();
                    sink.push(ExternNonPrimitiveParam {
                        param: InFile::new(self.func.source(self.db.upcast()).file_id, arg_ptr),
                    })
                }
            }

            let return_ty = sig.ret();
            if return_ty.as_struct().is_some() {
                let arg_ptr = fn_data
                    .type_ref_source_map()
                    .type_ref_syntax(*fn_data.ret_type())
                    .map(|ptr| ptr.syntax_node_ptr())
                    .unwrap();
                sink.push(ExternNonPrimitiveParam {
                    param: InFile::new(self.func.source(self.db.upcast()).file_id, arg_ptr),
                })
            }
        }
    }
}
