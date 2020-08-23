use crate::Config;
use mun_codegen::{CodeGenDatabase, CodeGenDatabaseStorage};
use mun_hir::{salsa, HirDatabase, Upcast};

/// A compiler database is a salsa database that enables increment compilation.
#[salsa::database(
    mun_hir::SourceDatabaseStorage,
    mun_hir::DefDatabaseStorage,
    mun_hir::HirDatabaseStorage,
    CodeGenDatabaseStorage
)]
pub struct CompilerDatabase {
    storage: salsa::Storage<Self>,
}

impl Upcast<dyn mun_hir::SourceDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn mun_hir::SourceDatabase {
        &*self
    }
}

impl Upcast<dyn mun_hir::DefDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn mun_hir::DefDatabase {
        &*self
    }
}

impl Upcast<dyn mun_hir::HirDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn mun_hir::HirDatabase {
        &*self
    }
}

impl Upcast<dyn CodeGenDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn CodeGenDatabase {
        &*self
    }
}

impl CompilerDatabase {
    /// Constructs a new database
    pub fn new(config: &Config) -> Self {
        let mut db = CompilerDatabase {
            storage: Default::default(),
        };

        // Set the initial configuration
        db.set_config(config);

        db
    }

    /// Applies the given configuration to the database
    pub fn set_config(&mut self, config: &Config) {
        self.set_target(config.target.clone());
        self.set_optimization_level(config.optimization_lvl);
    }
}

impl salsa::Database for CompilerDatabase {}
