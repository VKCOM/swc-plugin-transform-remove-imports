#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use swc_core::ecma::visit::visit_mut_pass;

use serde::Deserialize;
use swc_cached::regex::CachedRegex;
use swc_core::ecma::visit::VisitMutWith;
use swc_core::{
    common::util::take::Take,
    ecma::{ast::*, transforms::testing::test, visit::VisitMut},
};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(untagged)]
pub enum Matcher {
    #[default]
    None,
    Regex(CachedRegex),
    Multi(Vec<Matcher>),
}

impl Matcher {
    #[cfg(test)]
    pub fn new(input: &str) -> Result<Self> {
        let r = CachedRegex::new(input)?;

        Ok(Matcher::Regex(r))
    }

    pub fn matches(&self, text: &str) -> bool {
        match self {
            Matcher::None => false,

            Matcher::Regex(re) => re.is_match(text),
            Matcher::Multi(ref v) => {
                for m in v {
                    if m.matches(text) {
                        return true;
                    }
                }

                false
            }
        }
    }
}

#[derive(Debug, Default, Deserialize)]
enum Remove {
    #[default]
    None,

    /// Removing only side effects imports
    #[serde(rename = "effects")]
    Effects,
}

#[derive(Debug, Deserialize)]
pub struct TransformVisitor {
    /// A regular expression to match the imports that will be removed
    test: Matcher,

    /// Used with the test option
    #[serde(default)]
    remove: Remove,
}

impl TransformVisitor {
    fn is_need_remove_call(&mut self, call: &mut CallExpr) -> bool {
        if let Callee::Expr(expr) = &mut call.callee {
            if let Expr::Ident(call_name) = &mut **expr {
                if call_name.sym.ne("require") || call.args.is_empty() {
                    return false;
                }

                if let Expr::Lit(Lit::Str(arg)) = &*call.args[0].expr {
                    return self.test.matches(&arg.value);
                }
            }
        }

        false
    }
}

impl VisitMut for TransformVisitor {
    fn visit_mut_import_decl(&mut self, n: &mut ImportDecl) {
        if !self.test.matches(&n.src.value) {
            return;
        }

        match &self.remove {
            Remove::Effects => {
                if n.specifiers.is_empty() {
                    n.take();
                }
            }
            _ => {
                n.take();
            }
        }
    }

    fn visit_mut_var_declarator(&mut self, v: &mut VarDeclarator) {
        if let Some(expr) = &mut v.init {
            if let Expr::Call(call) = &mut **expr {
                if !self.is_need_remove_call(call) {
                    return;
                }

                match &self.remove {
                    Remove::Effects => {}
                    _ => {
                        v.name.take();
                    }
                }
            }
        }

        v.visit_mut_children_with(self);
    }

    fn visit_mut_var_declarators(&mut self, vars: &mut Vec<VarDeclarator>) {
        vars.visit_mut_children_with(self);

        vars.retain(|node| {
            // We want to remove the node, so we should return false.
            if node.name.is_invalid() {
                return false;
            }

            // Return true if we want to keep the node.
            true
        });
    }

    fn visit_mut_stmt(&mut self, s: &mut Stmt) {
        s.visit_mut_children_with(self);

        match s {
            Stmt::Decl(Decl::Var(var)) => {
                if var.decls.is_empty() {
                    // Variable declaration without declarator is invalid.
                    //
                    // After this, `s` becomes `Stmt::Empty`.
                    s.take();
                }
            }
            Stmt::Expr(expr) => {
                if let Expr::Call(call) = &mut *expr.expr {
                    if self.is_need_remove_call(call) {
                        s.take();
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        stmts.visit_mut_children_with(self);

        // We remove `Stmt::Empty` from the statement list.
        // This is optional, but it's required if you don't want extra `;` in output.
        stmts.retain(|s| {
            // We use `matches` macro as this match is trivial.
            !matches!(s, Stmt::Empty(..))
        });
    }

    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);

        // We do same thing here.
        stmts.retain(|s| match s {
            ModuleItem::ModuleDecl(ModuleDecl::Import(x)) => !x.src.is_empty(),
            ModuleItem::Stmt(Stmt::Empty(..)) => false,
            _ => true,
        });
    }
}

// TEST //

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("\\.(less|css)$").unwrap(),
        remove: Remove::None,
    }),
    example,
    r#"
import './index.less';
import './index.main.css';
import { Button } from 'uiw';
import { Select } from '@uiw/core';
"#
);

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("").unwrap(),
        remove: Remove::None,
    }),
    remove_all,
    r#"
import './index.less';
import './index.main.css';
import { Button } from 'uiw';
import { Select } from '@uiw/core';
"#
);

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("foo").unwrap(),
        remove: Remove::Effects,
    }),
    effects,
    r#"
import 'foo';
import Foo from 'foo';
"#
);

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("\\.(less|css)$").unwrap(),
        remove: Remove::None,
    }),
    require_example,
    r#"
require('./index.less');
require('./index.main.css');
const uiw = require('uiw');
const core = require('@uiw/core');
"#
);

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("").unwrap(),
        remove: Remove::None,
    }),
    require_remove_all,
    r#"
require('./index.less');
require('./index.main.css');
const uiw = require('uiw');
const core = require('@uiw/core');
const test = "test";
"#
);

test!(
    Default::default(),
    |_| visit_mut_pass(TransformVisitor {
        test: Matcher::new("foo").unwrap(),
        remove: Remove::Effects,
    }),
    require_effects,
    r#"
require('foo');
const foo = require('foo');
"#
);
