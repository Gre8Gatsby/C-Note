import 'package:flutter/material.dart';

class homeScreen extends StatefulWidget {
  const homeScreen({Key? key}) : super(key: key);

  @override
  State<homeScreen> createState() => _homeScreenState();
}

class _homeScreenState extends State<homeScreen> {
  @override
  Widget build(BuildContext context) {
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
    double ffem = fem * 0.97;
    return Container(
      width: double.infinity,
      child: Container(
        // login8tR (1:38)
        padding: EdgeInsets.fromLTRB(30*fem, 0*fem, 0*fem, 0*fem),
        width: double.infinity,
        decoration: BoxDecoration (
          color: Color(0xffffffff),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Container(
              // vector11hK (1:59)
              margin: EdgeInsets.fromLTRB(7*fem, 0*fem, 0*fem, 47*fem),
              width: 391*fem,
              height: 119*fem,
              child: Image.asset(
                'images/img1.png',
                width: 391*fem,
                height: 119*fem,
              ),
            ),
            CardButton(
              title: 'Take Live Note',
              onPressed: () {
                Navigator.pushNamed(context, '/liveNotes');
              },
            ),
            SizedBox(height: 16),
            CardButton(
              title: 'Get Call Summary',
              onPressed: () {
                Navigator.pushNamed(context, '/callSummary');
              },
            ),
            SizedBox(height: 270),
            Flexible(child:
            Container(
              margin: EdgeInsets.fromLTRB(51 * fem, 0, 0, 0),
              width: 352.86 * fem,
              height: 398.83 * fem,
              child: Align(
                child: Image.asset(
                  'images/img2.png',
                  width: 441.86 * fem,
                  height: 398.83 * fem,
                ),
              ),
            ),
            ),

          ],
        ),
      ),
    );
  }
}

class CardButton extends StatelessWidget {
  final String title;
  final VoidCallback onPressed;

  const CardButton({required this.title, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 4,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(10),
      ),
      color: Colors.blue, // Set the background color to blue
      child: InkWell(
        onTap: onPressed,
        borderRadius: BorderRadius.circular(10),
        child: Container(
          padding: EdgeInsets.symmetric(vertical: 16, horizontal: 24),
          child: Text(
            title,
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w600,
              color: Colors.white, // Set the text color to white
            ),
          ),
        ),
      ),
    );
  }
}
