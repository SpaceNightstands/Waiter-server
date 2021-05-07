import 'package:shelf/shelf.dart' show Middleware;
import 'ResponseJson.dart' show Response;
import 'Error.dart';

Middleware idempotencyCache(String key) {

  return (handler) => (request) async {
        return handler(request);
      };
}

class IdempotencyError extends Error {
  const IdempotencyError(String message) : super('IdempotencyCache', message);
}
