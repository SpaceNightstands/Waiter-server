import 'package:pedantic/pedantic.dart' show unawaited;
import 'package:shelf/shelf.dart' show Middleware;
import 'package:stash/stash_api.dart' show Cache;
import '../ResponseJson.dart' show ResponseJson;
import '../Error.dart';
import './Authentication.dart' show AuthToken;

Middleware idempotencyCache(Cache cache) {
  return (handler) => (request) async {
        if (request.context['jwt'] == null ||
            request.context['jwt'] is! AuthToken) {
          return ResponseJson.fromJson(500,
              body: IdempotencyError('The token is missing or invalid'));
        }
        final token = request.context['jwt']! as AuthToken;
        final key = '${token.subject}${token.idempotency}';

        if (await cache.containsKey(key)) {
          ResponseJson.okFromJson(
              IdempotencyError('Request already replied to'));
        }

        final response = await handler(request);
        unawaited(cache.put(key, null));
        return response;
      };
}

class IdempotencyError extends Error {
  const IdempotencyError(String message) : super('IdempotencyCache', message);
}
