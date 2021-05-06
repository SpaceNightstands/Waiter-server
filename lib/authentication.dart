import 'dart:io' show HttpHeaders;
import 'dart:convert' show utf8;
import 'package:jose/jose.dart'
    show JsonWebKey, JsonWebKeyStore, JsonWebToken, JoseException;
import 'package:shelf/shelf.dart' show Handler, Middleware, Response;

Middleware authentication(String key) {
  final keyBytes = utf8.encode(key);
  final keyBigint = keyBytes.fold(
      BigInt.zero, (BigInt bigint, byte) => (bigint << 8) | BigInt.from(byte));
  final jsonWebKey = JsonWebKey.symmetric(key: keyBigint);
  final jsonWebKeyStore = JsonWebKeyStore()..addKey(jsonWebKey);

  return (handler) => (request) async {
    final header = request.headers[HttpHeaders.authorizationHeader];

    if (header == null) {
      //TODO: return error
      return Response(400, body: 'error');
    }

    if (!header.startsWith(RegExp(r'[Bb]earer '))) {
      //TODO: return error
      return Response(400, body: 'error');
    }

    final serailizedToken = header.substring('bearer '.length).trim();
    final JsonWebToken token;
    try {
      token = await JsonWebToken.decodeAndVerify(
          serailizedToken, jsonWebKeyStore);
    } on JoseException catch (exception) {
      //TODO: return error
      return Response(400, body: exception.message);
    }

    //TODO: check expiration timestamp
    //TODO: check validity
    //TODO: add token to request.context

    return handler(request);
  };
}
