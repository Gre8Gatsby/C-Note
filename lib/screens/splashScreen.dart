import 'dart:async';

import 'package:flutter/material.dart';
import 'package:verbalyze/screens/homeScreen.dart';
//import 'package:untitled/screens/noteScreen.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({Key? key}) : super(key: key);

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  @override
  Widget build(BuildContext context) {
    Timer(Duration(seconds: 5),
            () => Navigator.pushReplacement(context, MaterialPageRoute(builder:
            (context) => homeScreen())));
    return initWidget();
  }
  initWidget(){
    return Scaffold(
      body: Stack(
        children: [
          Container(
            decoration: BoxDecoration(
                color: new Color(0xff3F51B5),
                gradient: LinearGradient(
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                    colors: [Colors.purple, Colors.blue])
            ),
          ),

          Center(
            child: Container(
              child: Image.asset("images/verbalyze.png"),

            ),

          )
        ],
      ),
    );
  }
}
