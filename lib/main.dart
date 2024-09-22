import 'package:flutter/material.dart';
import 'package:c_note/screens/splashScreen.dart';
import 'package:c_note/screens/noteScreen.dart';
import 'package:c_note/screens/summaryScreen.dart';
import 'package:c_note/services/background_call_service.dart' as bg_service;
import 'package:permission_handler/permission_handler.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Request necessary permissions
  await _requestPermissions();
  
  // Initialize and start the background service
  await bg_service.BackgroundCallService.initializeService();
  
  runApp(const MyApp());
}

Future<void> _requestPermissions() async {
  await Permission.contacts.request();
  await Permission.phone.request();
  // For call log, we use the phone permission
  // If you need more specific permissions, you might need to implement
  // platform-specific code or use a different package for call log access
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'C Note',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        scaffoldBackgroundColor: Colors.white, // Set the default background color to white
        appBarTheme: AppBarTheme(
          backgroundColor: Colors.white, // Set app bar background to white
          foregroundColor: Colors.black, // Set app bar text color to black for contrast
          elevation: 0, // Remove app bar shadow
        ),
      ),
      initialRoute: '/',
      routes: {
        '/': (context) => const SplashScreen(),
        '/note': (context) => const NoteScreen(),
        '/summary': (context) => const SummaryScreen(),
      },
    );
  }
}
