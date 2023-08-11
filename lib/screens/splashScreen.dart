// import 'dart:async';
//
// import 'package:flutter/material.dart';
// import 'package:verbalyze/screens/homeScreen.dart';
// //import 'package:untitled/screens/noteScreen.dart';
//
// class SplashScreen extends StatefulWidget {
//   const SplashScreen({Key? key}) : super(key: key);
//
//   @override
//   State<SplashScreen> createState() => _SplashScreenState();
// }
//
// class _SplashScreenState extends State<SplashScreen> {
//   @override
//   Widget build(BuildContext context) {
//     Timer(Duration(seconds: 5),
//             () => Navigator.pushReplacement(context, MaterialPageRoute(builder:
//             (context) => homeScreen())));
//     return initWidget();
//   }
//   initWidget(){
//     return Scaffold(
//       body: Stack(
//         children: [
//           Container(
//             decoration: BoxDecoration(
//                 color: new Color(0xff3F51B5),
//                 gradient: LinearGradient(
//                     begin: Alignment.topLeft,
//                     end: Alignment.bottomRight,
//                     colors: [Colors.purple, Colors.blue])
//             ),
//           ),
//
//           Center(
//             child: Container(
//               child: Image.asset("images/verbalyze.png"),
//
//             ),
//
//           )
//         ],
//       ),
//     );
//   }
// }
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:verbalyze/screens/homeScreen.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({Key? key}) : super(key: key);

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  @override
  void initState() {
    super.initState();
    Timer(Duration(seconds: 5), () => Navigator.pushReplacement(
        context, MaterialPageRoute(builder: (context) => homeScreen())));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          Container(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [Colors.purple, Colors.blue],
              ),
            ),
          ),
          Center(
            child: Container(
              width: 200,
              height: 200,
              decoration: BoxDecoration(
                color: Colors.white, // Blue color for the logo container
                borderRadius: BorderRadius.circular(100),
                boxShadow: [
                  BoxShadow(
                    color: Colors.blue.withOpacity(0.6),
                    blurRadius: 10,
                    spreadRadius: 5,
                  ),
                ],
              ),
              child: Center(
                child: Image.asset(
                  "images/verbalyze.png",
                  width: 120,
                  height: 120,
                ),
              ),
            ),
          )
        ],
      ),
    );
  }
}
