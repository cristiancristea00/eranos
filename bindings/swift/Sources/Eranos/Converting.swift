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

/// `value` as the 32-bit integer the library takes, or the error `refusal` makes, which is the
/// library's own for the value, for an integer outside that range.
///
/// Without this check, `Int32(_:)` would stop the app on such an integer before the library saw
/// it.
func checked32Bits(_ value: Int, _ refusal: () -> EranosError) throws(EranosError) -> Int32 {
    guard let narrow = Int32(exactly: value) else {
        throw refusal()
    }
    return narrow
}

/// What `make` returns, or a `DecodingError` that carries the library's refusal of the data
/// `decoder` holds, so that a decoded value is one the library accepts.
func decoded<Value>(_ decoder: any Decoder, _ make: () throws(EranosError) -> Value) throws -> Value {
    do {
        return try make()
    } catch {
        throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: error.detail, underlyingError: error))
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
