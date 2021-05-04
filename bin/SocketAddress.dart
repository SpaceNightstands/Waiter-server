import 'dart:io' show InternetAddress;

class SocketAddress {
  final dynamic address;
  final int port;

  const SocketAddress(this.address, this.port) : assert(address is String || address is InternetAddress) ;
}
