import 'dart:io' show Platform, InternetAddress, HttpHeaders;
import 'dart:isolate' show Isolate;
import 'package:shelf/shelf.dart' show Pipeline;
import 'package:shelf/shelf_io.dart' show serve;
import 'package:shelf_helpers/shelf_helpers.dart' show cors;
import 'package:dotenv/dotenv.dart' as dotenv;
import 'package:Waiter/ResponseJson.dart' show Response;
import 'package:Waiter/Authentication.dart';
import 'SocketAddress.dart';
import 'ServerConfig.dart';
import 'ConfigError.dart';

void main() async {
  dotenv.load();

  var isolateCount = dotenv.env['THREAD_COUNT'] == null
      ? Platform.numberOfProcessors
      : int.parse(dotenv.env['THREAD_COUNT']!);
  if (Platform.numberOfProcessors >= 8) {
    isolateCount ~/= 2;
  } else if (Platform.numberOfProcessors >= 4) {
    isolateCount = 4;
  }

  if (dotenv.env['JWT_SECRET'] == null) {
    throw const ConfigError('JWT_SECRET config variable not found');
  }
  final serverConfig = ServerConfig(
      SocketAddress(
          dotenv.env['SERVER_ADDRESS'] ?? InternetAddress.anyIPv4,
          dotenv.env['SERVER_PORT'] == null
              ? 8080
              : int.parse(dotenv.env['SERVER_PORT']!)),
      jwtKey: dotenv.env['JWT_SECRET']!);

  for (var i = 0; i < isolateCount - 1; ++i) {
    await Isolate.spawn(serverMain, serverConfig);
  }
  serverMain(serverConfig);
}

void serverMain(ServerConfig serverConfig) async {
  final handler = Pipeline()
      //CORS Middleware
      //TODO: make my own CORS middleware
      .addMiddleware(cors(headers: {
        'Access-Control-Allow-Origin': serverConfig.corsOrigin,
        'Access-Control-Allow-Methods': 'GET, PUT, DELETE',
        'Access-Control-Allow-Headers':
            '${HttpHeaders.contentTypeHeader} ${HttpHeaders.authorizationHeader}'
      }))
      //Authentication
      .addMiddleware(authentication('test'))
      //TODO: Idempotency cache
      .addHandler((req) => Response.okFromJson(req.context['jwt']));

  print(
      'Starting server on ${serverConfig.socket.address}:${serverConfig.socket.port}');
  await serve(handler, serverConfig.socket.address, serverConfig.socket.port,
      shared: true);
}
