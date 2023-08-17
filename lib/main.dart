import 'package:flutter/material.dart';
import 'package:verbalyze/screens/homeScreen.dart';
import 'package:verbalyze/screens/login.dart';
import 'package:verbalyze/screens/register.dart';
import 'package:verbalyze/screens/splashScreen.dart';
import 'package:verbalyze/screens/fileScreen.dart';
import 'package:verbalyze/screens/noteScreen.dart';
import 'package:verbalyze/screens/summaryScreen.dart';

import 'DrawerWidget.dart';


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
        '/liveNotes' : (context)  => noteScreen(),
        '/summary': (context) => summaryScreen(),
        '/register': (context) => RegisterPage(),
        '/login' : (context) => LoginPage(),
      },
      // builder: (context, child) {
      //   return Scaffold(
      //     appBar: AppBar(
      //       //title: Text('Cnote'),
      //     ),
      //     drawer: DrawerWidget(), // Use the DrawerWidget here
      //     body: child, // This will be the body of your app's screens
      //   );
      // },



    );
  }
}
