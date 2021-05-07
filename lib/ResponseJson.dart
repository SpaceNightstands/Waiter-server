import 'dart:convert' show json, Encoding;
import 'dart:io' show HttpHeaders, ContentType;
import 'package:shelf/shelf.dart' as shelf;

extension Response on shelf.Response {
  static shelf.Response fromJson(
    int statusCode,
    {
      Object? body,
      Object? Function(dynamic)? toEncodable,
      Map<String, Object>? headers,
      Encoding? encoding,
      Map<String, Object>? context
    }
  ) => shelf.Response(
    statusCode,
    body: json.encode(body, toEncodable: toEncodable),
    headers: headers?..putIfAbsent(HttpHeaders.contentTypeHeader, ()=>ContentType.json),
    encoding: encoding,
    context: context
  );

  static shelf.Response okFromJson(
    Object? body,
    {
      Object? Function(dynamic)? toEncodable,
      Map<String, Object>? headers,
      Encoding? encoding,
      Map<String, Object>? context
    }
  ) => shelf.Response.ok(
    json.encode(body, toEncodable: toEncodable),
    headers: headers?..putIfAbsent(HttpHeaders.contentTypeHeader, ()=>ContentType.json),
    encoding: encoding,
    context: context
  );
}
