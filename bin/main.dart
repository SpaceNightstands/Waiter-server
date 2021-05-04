import 'dart:io' show Platform, InternetAddress;
import 'dart:isolate' show Isolate;
import 'package:alfred/alfred.dart' show Alfred, cors;
import 'package:dotenv/dotenv.dart' as dotenv;
import 'SocketAddress.dart';
import 'ServerConfig.dart';

void main() async {
  dotenv.load();

  var isolateCount = dotenv.env['THREAD_COUNT'] == null
      ? Platform.numberOfProcessors
      : int.parse(dotenv.env['THREAD_COUNT'] /*!*/);
  if (Platform.numberOfProcessors >= 8) {
    isolateCount ~/= 2;
  } else if (Platform.numberOfProcessors >= 4) {
    isolateCount = 4;
  }

  final serverConfig = ServerConfig(
    SocketAddress(
      dotenv.env['SERVER_ADDRESS'] ?? InternetAddress.anyIPv4,
      dotenv.env['SERVER_PORT'] == null
          ? 8080
          : int.parse(dotenv.env['SERVER_PORT'] /*!*/)
    ),
  );

  for (var i = 0; i < isolateCount - 1; ++i) {
    await Isolate.spawn(serverMain, serverConfig);
  }
  serverMain(serverConfig);
}

void serverMain(ServerConfig serverConfig) async {
  final server = Alfred();

  server.get(
    '*',
    cors(
      headers: 'Authorization',
      methods: 'GET, PUT, DELETE',
      origin: serverConfig.corsOrigin
    )
  );

  server.get('*', (req, res) => 'Hello, World!');

  await server.listen(serverConfig.socket.port, serverConfig.socket.address, true);
}
