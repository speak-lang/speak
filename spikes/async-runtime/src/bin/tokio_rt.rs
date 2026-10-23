fn main() {
    async_runtime_spike::bench::main(async_runtime_spike::tokio_exec::TokioExec::new);
}
