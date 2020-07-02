use crate::utils::{match_def_path, paths, span_lint, span_lint_and_help};
use if_chain::if_chain;
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty;
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_clippy_lint! {
    /// **What it does:** Checks for comparisons with an address of a function item.
    ///
    /// **Why is this bad?** Function item address is not guaranteed to be unique and could vary
    /// between different code generation units. Furthermore different function items could have
    /// the same address after being merged together.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// type F = fn();
    /// fn a() {}
    /// let f: F = a;
    /// if f == a {
    ///     // ...
    /// }
    /// ```
    pub FN_ADDRESS_COMPARISONS,
    correctness,
    "comparison with an address of a function item"
}

declare_clippy_lint! {
    /// **What it does:** Checks for comparisons with an address of a trait vtable.
    ///
    /// **Why is this bad?** Comparing trait objects pointers compares an vtable addresses which
    /// are not guaranteed to be unique and could vary between different code generation units.
    /// Furthermore vtables for different types could have the same address after being merged
    /// together.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust,ignore
    /// let a: Rc<dyn Trait> = ...
    /// let b: Rc<dyn Trait> = ...
    /// if Rc::ptr_eq(&a, &b) {
    ///     ...
    /// }
    /// ```
    pub VTABLE_ADDRESS_COMPARISONS,
    correctness,
    "comparison with an address of a trait vtable"
}

declare_lint_pass!(UnnamedAddress => [FN_ADDRESS_COMPARISONS, VTABLE_ADDRESS_COMPARISONS]);

impl LateLintPass<'_> for UnnamedAddress {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &Expr<'_>) {
        fn is_comparison(binop: BinOpKind) -> bool {
            match binop {
                BinOpKind::Eq | BinOpKind::Lt | BinOpKind::Le | BinOpKind::Ne | BinOpKind::Ge | BinOpKind::Gt => true,
                _ => false,
            }
        }

        fn is_trait_ptr(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
            match cx.tables().expr_ty_adjusted(expr).kind {
                ty::RawPtr(ty::TypeAndMut { ty, .. }) => ty.is_trait(),
                _ => false,
            }
        }

        fn is_fn_def(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
            if let ty::FnDef(..) = cx.tables().expr_ty(expr).kind {
                true
            } else {
                false
            }
        }

        if_chain! {
            if let ExprKind::Binary(binop, ref left, ref right) = expr.kind;
            if is_comparison(binop.node);
            if is_trait_ptr(cx, left) && is_trait_ptr(cx, right);
            then {
                span_lint_and_help(
                    cx,
                    VTABLE_ADDRESS_COMPARISONS,
                    expr.span,
                    "comparing trait object pointers compares a non-unique vtable address",
                    None,
                    "consider extracting and comparing data pointers only",
                );
            }
        }

        if_chain! {
            if let ExprKind::Call(ref func, [ref _left, ref _right]) = expr.kind;
            if let ExprKind::Path(ref func_qpath) = func.kind;
            if let Some(def_id) = cx.qpath_res(func_qpath, func.hir_id).opt_def_id();
            if match_def_path(cx, def_id, &paths::PTR_EQ) ||
                match_def_path(cx, def_id, &paths::RC_PTR_EQ) ||
                match_def_path(cx, def_id, &paths::ARC_PTR_EQ);
            let ty_param = cx.tables().node_substs(func.hir_id).type_at(0);
            if ty_param.is_trait();
            then {
                span_lint_and_help(
                    cx,
                    VTABLE_ADDRESS_COMPARISONS,
                    expr.span,
                    "comparing trait object pointers compares a non-unique vtable address",
                    None,
                    "consider extracting and comparing data pointers only",
                );
            }
        }

        if_chain! {
            if let ExprKind::Binary(binop, ref left, ref right) = expr.kind;
            if is_comparison(binop.node);
            if cx.tables().expr_ty_adjusted(left).is_fn_ptr() &&
                cx.tables().expr_ty_adjusted(right).is_fn_ptr();
            if is_fn_def(cx, left) || is_fn_def(cx, right);
            then {
                span_lint(
                    cx,
                    FN_ADDRESS_COMPARISONS,
                    expr.span,
                    "comparing with a non-unique address of a function item",
                );
            }
        }
    }
}
