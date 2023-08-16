import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:verbalyze/utils.dart';

class RegisterPage extends StatefulWidget {
  @override
  _RegisterPageState createState() => _RegisterPageState();
}

class _RegisterPageState extends State<RegisterPage> {
  TextEditingController nameController = TextEditingController();
  TextEditingController passwordController = TextEditingController();
  TextEditingController emailController = TextEditingController();
  TextEditingController mobileController = TextEditingController();



  void _register() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    prefs.setString('name', nameController.text);
    prefs.setString('password', passwordController.text);
    prefs.setString('email', emailController.text);
    prefs.setString('mobile', mobileController.text);
    Navigator.pushReplacementNamed(context, '/login'); // Navigate to login page
  }

  @override
  Widget build(BuildContext context) {
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
    double ffem = fem * 0.97;
    return Scaffold(
      body: SingleChildScrollView(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              // vector1gWV (1:66)
              margin: EdgeInsets.fromLTRB(0 * fem, 0 * fem, 0 * fem, 12 * fem),
              width: 391 * fem,
              height: 119 * fem,
              child: Image.asset(
                'images/vector-1-Nm7.png',
                width: 391 * fem,
                height: 119 * fem,
              ),
            ),
            Text(
              // createaccountaLy (1:77)
              'Create account',
              textAlign: TextAlign.center,
              style: SafeGoogleFont(
                'Lato',
                fontSize: 30 * ffem,
                fontWeight: FontWeight.w700,
                height: 1.2 * ffem / fem,
                color: Color(0xff262626),
              ),
            ),
            SizedBox(height: 35),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: nameController,
                  decoration: InputDecoration(
                    labelText: 'Name',
                    hintText: 'Your Name',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.person),
                  )),
            ),
            SizedBox(height: 20),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: passwordController,
                  decoration: InputDecoration(
                    labelText: 'Password',
                    hintText: 'Your Password',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.password),
                  )),
            ),
            SizedBox(height: 20),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: emailController,
                  decoration: InputDecoration(
                    labelText: 'Email',
                    hintText: 'Your Email',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.alternate_email),
                  )),
            ),
            SizedBox(height: 20),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: mobileController,
                  decoration: InputDecoration(
                    labelText: 'Mobile',
                    hintText: 'Phone Number',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.phone),
                  )),
            ),
            SizedBox(height: 20),
             Container(
              width: 150,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(8.0),
              ),
              child: ElevatedButton(
                onPressed: _register,
                style: ElevatedButton.styleFrom(
                  padding: EdgeInsets.all(16.0),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(8.0),
                  ),
                ),
                child: Text(
                  'Register',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 17.0,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ),
            SizedBox(height: 10),
            TextButton(
              onPressed: () {
                Navigator.pushReplacementNamed(
                    context, '/login'); // Navigate to login page
              },
              child: Text('Already have an account? Log in'),
            ),

                // SizedBox(
                //   width: 352.86 * fem,
                //   height: 398.83 * fem,
                //   child: Image.asset(
                //     'images/vector-2-hDk.png',
                //     width: 641.86 * fem,
                //     height: 798.83 * fem,
                //   ),
                // ),


          ],
        ),
      ),
    );
  }
}
