import 'package:flutter/material.dart';
import 'package:verbalyze/screens/homeScreen.dart';
import 'package:verbalyze/screens/splashScreen.dart';
import 'package:verbalyze/screens/fileScreen.dart';
import 'package:verbalyze/screens/noteScreen.dart';


void main(){
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      home: SplashScreen(),
      routes: {
        '/home' : (context) => homeScreen(),
        '/callSummary'   : (context) => fileScreen(),
        '/liveNotes' : (context)  => noteScreen()
      },

    );
  }
}
