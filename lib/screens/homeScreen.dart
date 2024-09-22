import 'package:flutter/material.dart';
import 'package:share_plus/share_plus.dart';
import 'package:url_launcher/url_launcher.dart';


class homeScreen extends StatefulWidget {
  const homeScreen({super.key});

  @override
  State<homeScreen> createState() => _homeScreenState();
}

class _homeScreenState extends State<homeScreen> {
  @override
  Widget build(BuildContext context) {
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
    // double ffem = fem * 0.97;
    final drawer = Drawer(
      child: ListView(
        padding: EdgeInsets.zero,
        children: [
          const DrawerHeader(
            decoration: BoxDecoration(
              color: Colors.blue,
            ),
            child: Text('Drawer'),
          ),
          ListTile(
            title: const Text('Item 1'),
            onTap: () {},
          ),
          ListTile(
            title: const Text('Item 2'),
            onTap: () {},
          ),
        ],
      ),
    );

    return Scaffold(
      drawer: drawer,



    body:Container(

      //width: double.infinity,
      child: Container(
        // login8tR (1:38)
        padding: EdgeInsets.fromLTRB(9 * fem, 0 * fem, 0 * fem, 0 * fem),
        //width: double.infinity,
        decoration: const BoxDecoration(
          color: Color(0xffffffff),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Container(
              // vector11hK (1:59)
              margin: EdgeInsets.fromLTRB(7 * fem, 0 * fem, 0 * fem, 47 * fem),
              width: 400 * fem,
              height: 105 * fem,
              child: Image.asset(
                'images/img1.png',
                width: 391 * fem,
                height: 119 * fem,
              ),
            ),
            SizedBox(height: 20 * fem),
            GridView.count(
              crossAxisCount: 2, // Display two buttons per row
              crossAxisSpacing: 16,
              mainAxisSpacing: 16,
              childAspectRatio: 2, // Adjust this value to control the button's aspect ratio
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),

              children: [
                CardButton(
                  title: 'Take Live Note',
                  onPressed: () {
                    Navigator.pushNamed(context, '/liveNotes');
                  },
                ),
                CardButton(
                  title: 'Recorded call',
                  onPressed: () {
                    Navigator.pushNamed(context, '/callSummary');
                  },
                ),
                CardButton(
                  title: 'Get transcript',
                  onPressed: () {
                    Navigator.pushNamed(context, '/summary');
                  },
                ),
                CardButton(
                  title: 'Get summary',
                  onPressed: () {
                    Navigator.pushNamed(context, '/summary');
                  },
                ),
                CardButton(
                  title: 'Share App',
                  onPressed: () {
                    //Navigator.pushNamed(context, '/summary');
                    Share.share('Hi There, I am using the awesome app by Verbalyze where I can get text from voice spoken notes and generate summary of conferences , calls and classes.<our google form link to download the app>');
                  },
                ),
                CardButton(
                  title: 'Feedback',
                  onPressed: () async {
                    const feedbackUrl_ = 'https://www.verblayze.in';
                    final Uri feedbackUrl=Uri.parse(feedbackUrl_);
                    if (await canLaunchUrl(feedbackUrl)) {
                      await launchUrl(feedbackUrl);
                    } else {
                      throw 'Could not launch $feedbackUrl';
                    }
                  },
                ),

                // Add more buttons here if needed
              ],
            ),
            const SizedBox(height: 170),
            Flexible(
              child: Container(
                margin: EdgeInsets.fromLTRB(41 * fem, 0 * fem, 0 * fem, 0 * fem),
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
    )
    );
  }
}

class CardButton extends StatelessWidget {
  final String title;
  final VoidCallback onPressed;

  const CardButton({super.key, required this.title, required this.onPressed});

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
            gradient:  const LinearGradient (
              begin:  Alignment(-1, -1),
              end:  Alignment(1, 1),
              colors:  <Color>[Color(0xff23a5d6), Color(0xff623aa2)],
              stops:  <double>[0, 1],
            ),
          ),
          padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 24),
          child: Text(
            title,
            style: const TextStyle(
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
