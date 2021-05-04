import 'dart:io' show Platform, InternetAddress;
import 'dart:isolate' show Isolate;
import 'package:alfred/alfred.dart' show Alfred;
import 'SocketAddress.dart';

void main() async {
  var isolateCount = Platform.numberOfProcessors;
  if (Platform.numberOfProcessors >= 8) {
    isolateCount ~/= 2;
  } else if (Platform.numberOfProcessors >= 4) {
    isolateCount = 4;
  }

  final socket = SocketAddress(InternetAddress.loopbackIPv4, 8080);

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
