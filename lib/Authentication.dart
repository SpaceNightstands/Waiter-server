import 'dart:io' show HttpHeaders;
import 'dart:convert' show utf8;
import 'package:jose/jose.dart';
import 'package:shelf/shelf.dart' show Middleware;
import 'ResponseJson.dart' show Response;
import 'Error.dart';

Middleware authentication(String key) {
  final keyBigint = stringToBigInt(key);
  final jsonWebKey = JsonWebKey.symmetric(key: keyBigint);
  final jsonWebKeyStore = JsonWebKeyStore()..addKey(jsonWebKey);

  return (handler) => (request) async {
        final header = request.headers[HttpHeaders.authorizationHeader];

        if (header == null) {
          return Response.fromJson(400,
              body: AuthError(
                  '${HttpHeaders.authorizationHeader} header missing'));
        }

        if (!header.startsWith(RegExp(r'[Bb]earer '))) {
          return Response.fromJson(400,
              body: AuthError(
                  "${HttpHeaders.authorizationHeader} doesn't start with \"Bearer \""));
        }

        final serializedToken = header.substring('bearer '.length).trim();
        final JsonWebToken jwToken;
        try {
          jwToken = await JsonWebToken.decodeAndVerify(
              serializedToken, jsonWebKeyStore);
        } on JoseException catch (exception) {
          return Response.fromJson(400, body: AuthError(exception.message));
        }

        //TODO: check expiration timestamp
        //Constructor does schema validation
        final newContext = Map.of(request.context);
        try {
          newContext['jwt'] = AuthToken(jwToken.claims);
        } on AuthError catch (error) {
          return Response.fromJson(400, body: error);
        }

        return handler(request.change(context: newContext));
      };
}

BigInt stringToBigInt(String source) {
  final bytes = utf8.encode(source);
  final bigint = bytes.fold(
      BigInt.zero, (BigInt accumulator, byte) => (accumulator << 8) | BigInt.from(byte));
  return bigint;
}

class AuthToken {
  final String subject;
  final DateTime expiry;
  final String idempotency;

  const AuthToken.fromParts(this.subject, this.expiry, this.idempotency);

  factory AuthToken(JsonWebTokenClaims jwt) {
    if (jwt.subject != null &&
        jwt.expiry != null &&
        jwt['idempotency'] != null) {
      return AuthToken.fromParts(jwt.subject!, jwt.expiry!, jwt['idempotency']);
    } else {
      throw const AuthError('Failed JWT validation');
    }
  }

  Map<String, dynamic> toJson() =>
      {'subject': subject, 'expiry': expiry, 'idempotency': idempotency};
}

class AuthError extends Error {
  const AuthError(String message) : super('Authorization', message);
}