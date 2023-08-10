import 'package:flutter/material.dart';

class homeScreen extends StatefulWidget {
  const homeScreen({Key? key}) : super(key: key);

  @override
  State<homeScreen> createState() => _homeScreenState();
}

class _homeScreenState extends State<homeScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Home Page')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              child: Text('Take Live Note'),
              onPressed: () {
                Navigator.pushNamed(context, '/liveNotes');
              },

            ),
            ElevatedButton(
              child: Text('Get call summary'),
              onPressed: () {
                Navigator.pushNamed(context, '/callSummary');
              },

            ),
          ],
        ),
      ),
    );
  }
}

