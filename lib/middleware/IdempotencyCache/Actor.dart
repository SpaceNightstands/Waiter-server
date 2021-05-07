import 'dart:async' show FutureOr;
import 'package:actors/actors.dart' show Actor, Handler;

void spawnActor() {}

class _IdempotencyCache implements Handler<dynamic, dynamic> {
  final Set<String> idempCache = {};
  //final Map<String, List<int>> idempCache = {};

  @override
  FutureOr<dynamic> handle(dynamic message) {
    if (message is _RequestMessage) {
    } else if (message is _UpdateMessage) {}
  }

  _IdempotencyCache() {
    //TODO: Schedule idempCache clearance
  }
}

class _RequestMessage {
  final String key;

  const _RequestMessage(this.key);
}

class _UpdateMessage {
  final String key;
  //final List<int> chunk;

  const _UpdateMessage(this.key);
}
