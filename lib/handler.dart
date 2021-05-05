import 'dart:async' show FutureOr;
import 'package:alfred/alfred.dart' show HttpRequest, HttpResponse;

typedef Handler = FutureOr<dynamic> Function(HttpRequest, HttpResponse);
