use crate::analysis::typer::Typer;
use crate::analysis::typer::EXPR_RESULT_USED;
use crate::error::Error;
use crate::mir::{Assignment, MirNode, MirStmt, MirStmtKind};
use crate::syntax::ast::{AssignmentOp, Node, NodeType, Stmt, StmtKind};
use std::ops::Deref;
use std::rc::Rc;

#[allow(unused)]
macro_rules! with_state {
    ($typer:expr, $state:expr, $body:tt) => {{
        let old_state = $typer.state;
        $typer.state |= $state;
        let res = $body;
        $typer.state = old_state;
        res
    }};
}

impl<'src> Typer<'src> {
    pub(crate) fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<Rc<MirStmt>, Error> {
        self.resolve_stmt_inner(stmt, false)
    }

    pub(super) fn resolve_stmt_inner(
        &mut self,
        stmt: &Stmt,
        top_level: bool,
    ) -> Result<Rc<MirStmt>, Error> {
        println!("Resolving Stmt {}", stmt.kind().name());

        match stmt.kind() {
            StmtKind::Expr(expr) => {
                let old_state = self.state;
                self.state &= !EXPR_RESULT_USED;
                let expr = self.resolve_expr(expr.as_ref(), None)?;
                self.state = old_state;
                let position = expr.position();
                let ty = expr.ty();
                Ok(Rc::new(MirStmt::new(MirStmtKind::Expr(expr), position, ty)))
            }
            StmtKind::Item(item) => {
                let entity = if top_level {
                    self.resolve_top_level_item(item.as_ref())?
                } else {
                    self.resolve_item(item.as_ref())?
                };

                let position = item.position();
                // let ty = entity.deref().borrow().ty();
                Ok(Rc::new(MirStmt::new(
                    MirStmtKind::Item(entity),
                    position,
                    self.type_map.get_unit(),
                )))
            }
            StmtKind::Assignment { op, lvalue, rhs } => {
                let (entity, mir_lvalue) = self.resolve_expr_to_entity(lvalue.as_ref())?;
                // let lvalue_type = mir_lvalue.ty();
                let mutability = mir_lvalue.inner().meta();
                if !mutability.mutable {
                    let err = Error::immutable_entity(entity.deref().borrow().name());
                    return Err(err.with_position(lvalue.position()));
                }
                match op {
                    AssignmentOp::Assign => {
                        let lvalue_type = mir_lvalue.ty();
                        let rhs = self.resolve_expr(rhs.as_ref(), Some(lvalue_type))?;
                        let assignment = Assignment {
                            op: *op,
                            lvalue: entity,
                            rhs,
                        };

                        Ok(Rc::new(MirStmt::new(
                            MirStmtKind::Assignment(assignment),
                            stmt.position(),
                            self.type_map.get_unit(),
                        )))
                    }
                    _ => todo!("Assignment operator {} is not implemented", op),
                }
            }
            StmtKind::Empty => unreachable!(),
        }
    }
}
