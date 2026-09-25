internal import EranosGenerated

/// Runs `body`, which calls the generated code, and reports what it throws as an ``EranosError``.
///
/// The generated code throws untyped errors: the library's error, or one of UniFFI's own, which
/// the library's calls never cause. Every call of the package goes through this function or its
/// asynchronous twin, so the one conversion lives here.
func converting<Value>(_ body: () throws -> Value) throws(EranosError) -> Value {
    do {
        return try body()
    } catch let error as EranosGenerated.EranosError {
        throw EranosError(error)
    } catch {
        throw .unexpected(detail: String(describing: error))
    }
}

/// Runs `body`, which calls the generated code with a cancellation, and reports what it throws
/// as an ``EranosError``.
///
/// Cancelling the task cancels the cancellation, so the library's call stops at its next yield,
/// or at once while it waits for the group's lock, and throws ``EranosError/cancelled(detail:)``,
/// leaving the group as it was.
func converting<Value: Sendable>(
    _ body: @Sendable (EranosGenerated.Cancellation) async throws -> Value
) async throws(EranosError) -> Value {
    let cancellation = EranosGenerated.Cancellation()
    do {
        return try await withTaskCancellationHandler {
            try await body(cancellation)
        } onCancel: {
            cancellation.cancel()
        }
    } catch let error as EranosGenerated.EranosError {
        throw EranosError(error)
    } catch {
        throw .unexpected(detail: String(describing: error))
    }
}
