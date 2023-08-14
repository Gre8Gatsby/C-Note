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
   // double ffem = fem * 0.97;
    return Container(
      width: double.infinity,
      child: Container(
        // login8tR (1:38)
        padding: EdgeInsets.fromLTRB(10 * fem, 0 * fem, 0 * fem, 0 * fem),
        width: double.infinity,
        decoration: BoxDecoration(
          color: Color(0xffffffff),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Container(
              // vector11hK (1:59)
              margin: EdgeInsets.fromLTRB(7 * fem, 0  * fem, 0 * fem, 47 * fem),
              width: 391 * fem,
              height: 119 * fem,
              child: Image.asset(
                'images/img1.png',
                width: 391 * fem,
                height: 119 * fem,
              ),
            ),
            SizedBox(height: 20 * fem),
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
            SizedBox(height: 16),
            CardButton(
              title: 'Get text files',
              onPressed: () {
                Navigator.pushNamed(context, '/summary');
              },
            ),
            SizedBox(height: 170),
            Flexible(
              child: Container(
                margin: EdgeInsets.fromLTRB(41 * fem, 0 *fem, 0 * fem, 0 * fem),
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
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
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
          alignment: Alignment.center,
          decoration: BoxDecoration (
            borderRadius:  BorderRadius.circular(12*fem),
            gradient:  LinearGradient (
              begin:  Alignment(-1, -1),
              end:  Alignment(1, 1),
              colors:  <Color>[Color(0xff23a5d6), Color(0xff623aa2)],
              stops:  <double>[0, 1],
            ),
          ),
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
