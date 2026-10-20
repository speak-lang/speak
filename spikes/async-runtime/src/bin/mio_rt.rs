fn main() {
    async_runtime_spike::bench::main(async_runtime_spike::mio_exec::MioExec::new);
}
