import 'dart:io' show Platform, InternetAddress;
import 'dart:isolate' show Isolate;
import 'package:alfred/alfred.dart' show Alfred;
import 'package:dotenv/dotenv.dart' as dotenv;
import 'SocketAddress.dart';

void main() async {
  dotenv.load();

  var isolateCount = Platform.numberOfProcessors;
  if (Platform.numberOfProcessors >= 8) {
    isolateCount ~/= 2;
  } else if (Platform.numberOfProcessors >= 4) {
    isolateCount = 4;
  }

  
  final socket = SocketAddress(
    dotenv.env['SERVER_ADDRESS'] ?? InternetAddress.anyIPv4,
    dotenv.env['SERVER_PORT'] == null ? 8080 : int.tryParse(dotenv.env['SERVER_PORT'])
  );

  for (var i = 0; i < isolateCount - 1; ++i) {
    await Isolate.spawn(serverMain, socket);
  }
  serverMain(socket);
}

void serverMain(SocketAddress address) async {
  final server = Alfred();

  server.get('*', (request, response) => 'Hello, World!');

  await server.listen(address.port, address.address, true);
}
