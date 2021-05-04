import 'SocketAddress.dart';

class StartupData {
  final SocketAddress socket;
  final String corsOrigin;

  const StartupData(this.socket, {this.corsOrigin = '*'});
}
