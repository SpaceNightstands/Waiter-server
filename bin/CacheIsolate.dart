import 'dart:async' show FutureOr, Timer;
import 'dart:isolate' show ReceivePort, SendPort;
import 'package:stream_channel/isolate_channel.dart';

void idempotencyCache(ReceivePort receivePort) async {
  final SendPort sendPort = await receivePort.first;
  final channel = IsolateChannel(receivePort, sendPort);

  final Set<String> idempCache = {};
  late final void Function() clearer;
  clearer = () {
    idempCache.clear();
    Timer(_timeUntilMidnight(), clearer);
  };
  Timer(_timeUntilMidnight(), clearer);

  await channel.stream.forEach((message) {
    if (message is String) {
    } else {
      print('$message, of type ${message.runtimeType}');
    }
  });
}

Duration _timeUntilMidnight() {
  final now = DateTime.now();
  return now.difference(
      //Midnight
      DateTime(now.year, now.month, now.day + 1));
}

