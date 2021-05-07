import 'package:shelf/shelf.dart' show Middleware;
import '../ResponseJson.dart' show ResponseJson;
import '../Error.dart';

Middleware subjectFilter(List<String> key) {
  return (handler) => (request) async {
        return handler(request);
      };
}

class FilterError extends Error {
  const FilterError(String message) : super('SubFilter', message);
}
