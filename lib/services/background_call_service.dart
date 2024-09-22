import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:flutter_background_service_android/flutter_background_service_android.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:call_log/call_log.dart';
import 'package:contacts_service/contacts_service.dart';
import 'package:flutter_sound/flutter_sound.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:path_provider/path_provider.dart';
import 'gpt_service.dart';

class BackgroundCallService {
  static FlutterSoundRecorder? _recorder;
  static String? _recordingPath;

  static Future<void> initializeService() async {
    final service = FlutterBackgroundService();

    await service.configure(
      androidConfiguration: AndroidConfiguration(
        onStart: onStart,
        autoStart: true,
        isForegroundMode: true,
      ),
      iosConfiguration: IosConfiguration(
        autoStart: true,
        onForeground: onStart,
        onBackground: onIosBackground,
      ),
    );

    service.startService();
  }

  @pragma('vm:entry-point')
  static Future<bool> onIosBackground(ServiceInstance service) async {
    WidgetsFlutterBinding.ensureInitialized();
    return true;
  }

  @pragma('vm:entry-point')
  static void onStart(ServiceInstance service) async {
    WidgetsFlutterBinding.ensureInitialized();

    if (service is AndroidServiceInstance) {
      service.on('setAsForeground').listen((event) {
        service.setAsForegroundService();
      });

      service.on('setAsBackground').listen((event) {
        service.setAsBackgroundService();
      });
    }

    service.on('stopService').listen((event) {
      service.stopSelf();
    });

    _recorder = FlutterSoundRecorder();
    await _recorder!.openRecorder();

    Timer.periodic(Duration(seconds: 10), (timer) async {
      await checkForNewCalls(service);
    });
  }

  static Future<void> checkForNewCalls(ServiceInstance service) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final lastCheckedTimestamp = prefs.getInt('lastCheckedTimestamp') ?? 0;

      final now = DateTime.now();
      final Iterable<CallLogEntry> entries = await CallLog.get();

      for (var entry in entries) {
        if (entry.timestamp! > lastCheckedTimestamp) {
          final contactName = await getContactName(entry.number ?? '');
          await _startRecording();
          // Simulate recording for 10 seconds
          await Future.delayed(Duration(seconds: 10));
          final recordingFile = await _stopRecording();
          final transcript = "Simulated transcript for call with $contactName"; // In a real app, you'd get this from speech recognition
          final summary = await GPTService.summarizeCall(transcript);

          // Store the call note
          await storeCallNote(contactName, entry.number ?? '', DateTime.fromMillisecondsSinceEpoch(entry.timestamp!), summary, recordingFile);

          service.invoke(
            'update',
            {
              'current_date': DateTime.now().toIso8601String(),
              'call': {
                'name': contactName,
                'number': entry.number,
                'timestamp': entry.timestamp,
                'summary': summary,
              },
            },
          );
        }
      }

      await prefs.setInt('lastCheckedTimestamp', now.millisecondsSinceEpoch);
    } catch (e) {
      print('Error in checkForNewCalls: $e');
    }
  }

  static Future<String> getContactName(String phoneNumber) async {
    try {
      final Iterable<Contact> contacts = await ContactsService.getContacts();
      for (var contact in contacts) {
        for (var phone in contact.phones ?? []) {
          if (phone.value == phoneNumber) {
            return contact.displayName ?? 'Unknown';
          }
        }
      }
    } catch (e) {
      print('Error getting contact name: $e');
    }
    return 'Unknown';
  }

  static Future<void> storeCallNote(String contactName, String phoneNumber, DateTime timestamp, String summary, String recordingPath) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final key = '${contactName}_${timestamp.toIso8601String()}';
      final value = {
        'phoneNumber': phoneNumber,
        'timestamp': timestamp.toIso8601String(),
        'summary': summary,
        'recordingPath': recordingPath,
      };
      await prefs.setString(key, value.toString());
    } catch (e) {
      print('Error storing call note: $e');
    }
  }

  static Future<void> _startRecording() async {
    try {
      await Permission.microphone.request();
      await Permission.storage.request();

      final directory = await getApplicationDocumentsDirectory();
      _recordingPath = '${directory.path}/recording_${DateTime.now().millisecondsSinceEpoch}.aac';

      await _recorder!.startRecorder(toFile: _recordingPath);
    } catch (e) {
      print('Error starting recording: $e');
    }
  }

  static Future<String> _stopRecording() async {
    try {
      await _recorder!.stopRecorder();
      return _recordingPath ?? '';
    } catch (e) {
      print('Error stopping recording: $e');
      return '';
    }
  }
}