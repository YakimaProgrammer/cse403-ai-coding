use gloo_worker::Registrable;
use crate::app::SolverWorker;

mod app;
mod solver;
mod csv_parser;

fn main() {
    SolverWorker::registrar().register();
}
