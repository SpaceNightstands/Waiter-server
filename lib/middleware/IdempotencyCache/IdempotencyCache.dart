import 'package:shelf/shelf.dart' show Middleware;
import '../../ResponseJson.dart' show ResponseJson;
import '../../Error.dart';

Middleware idempotencyCache() {
  return (handler) => (request) async {
        return handler(request);
      };
}

class IdempotencyError extends Error {
  const IdempotencyError(String message) : super('IdempotencyCache', message);
}
