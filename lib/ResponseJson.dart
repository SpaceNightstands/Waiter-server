import 'dart:convert' show json, Encoding;
import 'dart:io' show HttpHeaders, ContentType;
import 'package:shelf/shelf.dart' show Response;

extension ResponseJson on Response {
  static Response fromJson(
    int statusCode,
    {
      Object? body,
      Object? Function(dynamic)? toEncodable,
      Map<String, Object>? headers,
      Encoding? encoding,
      Map<String, Object>? context
    }
  ) => Response(
    statusCode,
    body: json.encode(body, toEncodable: toEncodable),
    headers: headers?..putIfAbsent(HttpHeaders.contentTypeHeader, ()=>ContentType.json),
    encoding: encoding,
    context: context
  );

  static Response okFromJson(
    Object? body,
    {
      Object? Function(dynamic)? toEncodable,
      Map<String, Object>? headers,
      Encoding? encoding,
      Map<String, Object>? context
    }
  ) => Response.ok(
    json.encode(body, toEncodable: toEncodable),
    headers: headers?..putIfAbsent(HttpHeaders.contentTypeHeader, ()=>ContentType.json),
    encoding: encoding,
    context: context
  );
}
