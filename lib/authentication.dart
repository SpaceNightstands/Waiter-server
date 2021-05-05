import 'dart:convert' show utf8;
import 'package:jose/jose.dart'
    show JsonWebKey, JsonWebKeyStore, JsonWebToken, JoseException;
import './handler.dart';

Handler authentication(String key) {
  final keyBytes = utf8.encode(key);
  final keyBigint = keyBytes.fold(
      BigInt.zero, (BigInt bigint, byte) => (bigint << 8) | BigInt.from(byte));
  final jsonWebKey = JsonWebKey.symmetric(key: keyBigint);
  final jsonWebKeyStore = JsonWebKeyStore()..addKey(jsonWebKey);

  return (request, response) async {
    final header = request.headers.value('Authorization');

    if (header == null) {
      //TODO: return error
      return 'error';
    }

    if (!header.startsWith(RegExp(r'[Bb]earer '))) {
      //TODO: return error
      return 'error';
    }

    final serailizedToken = header.substring('bearer '.length).trim();
    final token;
    try {
      token =
          await JsonWebToken.decodeAndVerify(serailizedToken, jsonWebKeyStore);
    } on JoseException /*catch(exception)*/ {
      //TODO: return error
      return 'error';
    }
  };
}
